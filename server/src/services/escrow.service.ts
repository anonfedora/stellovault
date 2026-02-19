import { contracts } from "../config/contracts";
import { NotFoundError, ValidationError } from "../config/errors";
import contractService from "./contract.service";
import { prisma } from "./database.service";
import websocketService from "./websocket.service";

const ESCROW_STATUSES = new Set(["PENDING", "ACTIVE", "COMPLETED", "DISPUTED", "EXPIRED"]);
const ESCROW_ALLOWED_TRANSITIONS: Record<EscrowStatus, ReadonlySet<EscrowStatus>> = {
    PENDING: new Set(["PENDING", "ACTIVE", "COMPLETED", "DISPUTED", "EXPIRED"]),
    ACTIVE: new Set(["ACTIVE", "COMPLETED", "DISPUTED", "EXPIRED"]),
    COMPLETED: new Set(["COMPLETED"]),
    DISPUTED: new Set(["DISPUTED", "ACTIVE", "COMPLETED", "EXPIRED"]),
    EXPIRED: new Set(["EXPIRED"]),
};
const MIN_PAGE = 1;
const DEFAULT_PAGE = 1;
const DEFAULT_LIMIT = 20;
const MAX_LIMIT = 100;

type EscrowStatus = "PENDING" | "ACTIVE" | "COMPLETED" | "DISPUTED" | "EXPIRED";

interface CreateEscrowRequest {
    buyerId?: string;
    sellerId?: string;
    amount?: string | number;
    assetCode?: string;
    expiresAt?: string | Date;
}

interface EscrowListQuery {
    buyerId?: string;
    sellerId?: string;
    status?: string;
    page?: string | number;
    limit?: string | number;
}

interface EscrowEventPayload {
    escrowId?: string;
    status?: string;
    stellarTxHash?: string;
}

function parsePositiveAmount(value: string | number | undefined, fieldName: string): number {
    const amount = Number(value);
    if (!Number.isFinite(amount) || amount <= 0) {
        throw new ValidationError(`${fieldName} must be a positive number`);
    }
    return amount;
}

function parseDate(value: string | Date | undefined, fieldName: string): Date {
    if (!value) {
        throw new ValidationError(`${fieldName} is required`);
    }

    const parsed = value instanceof Date ? value : new Date(value);
    if (Number.isNaN(parsed.getTime())) {
        throw new ValidationError(`${fieldName} must be a valid date`);
    }
    return parsed;
}

function normalizeStatus(value: string | undefined, fieldName: string): EscrowStatus {
    const status = value?.trim().toUpperCase();
    if (!status || !ESCROW_STATUSES.has(status)) {
        throw new ValidationError(
            `${fieldName} must be one of: PENDING, ACTIVE, COMPLETED, DISPUTED, EXPIRED`
        );
    }
    return status as EscrowStatus;
}

function coercePage(value: string | number | undefined): number {
    const page = Number(value ?? DEFAULT_PAGE);
    if (!Number.isFinite(page) || page < MIN_PAGE) return DEFAULT_PAGE;
    return Math.floor(page);
}

function coerceLimit(value: string | number | undefined): number {
    const limit = Number(value ?? DEFAULT_LIMIT);
    if (!Number.isFinite(limit) || limit < 1) return DEFAULT_LIMIT;
    return Math.min(MAX_LIMIT, Math.floor(limit));
}

function ensureBase64(value: string): string {
    if (/^[A-Za-z0-9+/]+={0,2}$/.test(value) && value.length % 4 === 0) {
        return value;
    }
    return Buffer.from(value).toString("base64");
}

export class EscrowService {
    private timeoutTimer: NodeJS.Timeout | null = null;

    constructor() {
        this.timeoutDetector();
    }

    async createEscrow(payload: CreateEscrowRequest) {
        const buyerId = payload.buyerId?.trim();
        const sellerId = payload.sellerId?.trim();
        if (!buyerId) throw new ValidationError("buyerId is required");
        if (!sellerId) throw new ValidationError("sellerId is required");
        if (buyerId === sellerId) {
            throw new ValidationError("buyerId and sellerId must be different");
        }

        const amount = parsePositiveAmount(payload.amount, "amount");
        const expiresAt = parseDate(payload.expiresAt, "expiresAt");
        if (expiresAt.getTime() <= Date.now()) {
            throw new ValidationError("expiresAt must be in the future");
        }

        const db: any = prisma;
        const users = await db.user.findMany({
            where: { id: { in: [buyerId, sellerId] } },
            select: { id: true },
        });
        if (users.length !== 2) {
            throw new ValidationError("buyerId or sellerId does not exist");
        }

        const escrowContractId = contracts.escrow?.trim();
        if (!escrowContractId) {
            throw new Error("Escrow contract ID not configured: contracts.escrow is empty");
        }

        const unsignedXdr = await contractService.buildContractInvokeXDR(
            escrowContractId,
            "create_escrow",
            [buyerId, sellerId, amount.toString(), payload.assetCode || "USDC", expiresAt.toISOString()]
        );

        const escrow = await db.escrow.create({
            data: {
                buyerId,
                sellerId,
                amount: amount.toString(),
                assetCode: payload.assetCode || "USDC",
                status: "PENDING",
                expiresAt,
            },
        });

        return {
            escrowId: escrow.id,
            xdr: ensureBase64(unsignedXdr),
        };
    }

    async getEscrow(id: string) {
        const db: any = prisma;
        const escrow = await db.escrow.findUnique({
            where: { id },
            include: { buyer: true, seller: true },
        });

        if (!escrow) {
            throw new NotFoundError("Escrow not found");
        }
        return escrow;
    }

    async listEscrows(query: EscrowListQuery) {
        const db: any = prisma;
        const page = coercePage(query.page);
        const limit = coerceLimit(query.limit);
        const skip = (page - 1) * limit;

        const where: Record<string, string> = {};
        if (query.buyerId?.trim()) where.buyerId = query.buyerId.trim();
        if (query.sellerId?.trim()) where.sellerId = query.sellerId.trim();
        if (query.status?.trim()) where.status = normalizeStatus(query.status, "status");

        const [items, total] = await Promise.all([
            db.escrow.findMany({
                where,
                include: { buyer: true, seller: true },
                orderBy: { createdAt: "desc" },
                skip,
                take: limit,
            }),
            db.escrow.count({ where }),
        ]);

        const totalPages = Math.max(1, Math.ceil(total / limit));
        return {
            items,
            pagination: {
                page,
                limit,
                total,
                totalPages,
            },
        };
    }

    async processEscrowEvent(event: EscrowEventPayload) {
        const escrowId = event.escrowId?.trim();
        if (!escrowId) {
            throw new ValidationError("escrowId is required");
        }
        const status = normalizeStatus(event.status, "status");

        const db: any = prisma;
        const existing = await db.escrow.findUnique({ where: { id: escrowId } });
        if (!existing) {
            throw new NotFoundError("Escrow not found");
        }
        const allowedNextStatuses = ESCROW_ALLOWED_TRANSITIONS[existing.status as EscrowStatus];
        if (!allowedNextStatuses || !allowedNextStatuses.has(status)) {
            throw new ValidationError(
                `Illegal escrow status transition: ${existing.status} -> ${status}`
            );
        }

        const updateResult = await db.escrow.updateMany({
            where: { id: escrowId, status: existing.status },
            data: {
                status,
                ...(event.stellarTxHash ? { stellarTxHash: event.stellarTxHash } : {}),
            },
        });
        if (updateResult.count !== 1) {
            throw new ValidationError(
                "Escrow status changed concurrently. Please retry with latest state."
            );
        }

        const updated = await db.escrow.findUnique({
            where: { id: escrowId },
            include: { buyer: true, seller: true },
        });
        if (!updated) {
            throw new NotFoundError("Escrow not found");
        }

        if (existing.status !== updated.status) {
            websocketService.broadcastEscrowUpdated(updated.id, updated.status);
        }

        return updated;
    }

    timeoutDetector() {
        if (this.timeoutTimer) {
            return;
        }

        this.timeoutTimer = setInterval(async () => {
            try {
                const db: any = prisma;
                const overdueEscrows = await db.escrow.findMany({
                    where: {
                        status: "ACTIVE",
                        expiresAt: { lt: new Date() },
                    },
                    select: { id: true },
                });

                if (overdueEscrows.length === 0) {
                    return;
                }

                await db.escrow.updateMany({
                    where: {
                        status: "ACTIVE",
                        id: { in: overdueEscrows.map((item: { id: string }) => item.id) },
                    },
                    data: { status: "EXPIRED" },
                });

                const expiredEscrows = await db.escrow.findMany({
                    where: {
                        id: { in: overdueEscrows.map((item: { id: string }) => item.id) },
                        status: "EXPIRED",
                    },
                    select: { id: true },
                });

                for (const escrow of expiredEscrows) {
                    websocketService.broadcastEscrowUpdated(escrow.id, "EXPIRED");
                }
            } catch (error) {
                console.error("Escrow timeout detector failed:", error);
            }
        }, 60_000);

        this.timeoutTimer.unref();
    }
}

export default new EscrowService();

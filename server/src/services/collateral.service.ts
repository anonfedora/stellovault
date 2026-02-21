import { ConflictError, NotFoundError, ValidationError } from "../config/errors";
import { prisma } from "./database.service";
import { CollateralStatus } from "@prisma/client";
import { PrismaClientKnownRequestError } from "@prisma/client/runtime/library";

const UUID_REGEX = /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/;

interface CreateCollateralRequest {
    escrowId: string;
    assetCode?: string;
    amount: string | number;
    metadataHash: string;
}

interface CollateralListQuery {
    escrowId?: string;
    status?: string;
    page?: string | number;
    limit?: string | number;
}

const COLLATERAL_STATUSES = new Set(["PENDING", "DEPOSITED"]);
const MIN_PAGE = 1;
const DEFAULT_PAGE = 1;
const DEFAULT_LIMIT = 20;
const MAX_LIMIT = 100;

function parsePositiveAmount(value: string | number | undefined, fieldName: string): number {
    const amount = Number(value);
    if (!Number.isFinite(amount) || amount <= 0) {
        throw new ValidationError(`${fieldName} must be a positive number`);
    }
    return amount;
}

function normalizeStatus(value: string | undefined, fieldName: string): CollateralStatus {
    const status = value?.trim().toUpperCase();
    if (!status || !COLLATERAL_STATUSES.has(status)) {
        throw new ValidationError(
            `${fieldName} must be one of: PENDING, DEPOSITED`
        );
    }
    return status as CollateralStatus;
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

export class CollateralService {
    private indexerTimer: NodeJS.Timeout | null = null;
    private polling = false;
    
    constructor() {
        // Will be started explicitly by app.ts instead, or we can auto-start
    }

    async createCollateral(payload: CreateCollateralRequest) {
        const escrowId = payload.escrowId?.trim();
        const metadataHash = payload.metadataHash?.trim();
        
        if (!escrowId) throw new ValidationError("escrowId is required");
        if (!metadataHash) throw new ValidationError("metadataHash is required");

        const amount = parsePositiveAmount(payload.amount, "amount");

        const escrow = await prisma.escrow.findUnique({
            where: { id: escrowId },
            select: { id: true },
        });

        if (!escrow) {
            throw new ValidationError("escrowId does not exist");
        }

        try {
            const collateral = await prisma.collateral.create({
                data: {
                    escrowId,
                    amount: amount.toString(),
                    assetCode: payload.assetCode || "USDC",
                    metadataHash,
                    status: "PENDING",
                },
            });
            return collateral;
        } catch (error) {
            if (error instanceof PrismaClientKnownRequestError && error.code === "P2002") {
                throw new ConflictError("Collateral with this metadataHash already exists");
            }
            throw error;
        }
    }

    async getCollateralById(id: string) {
        const trimmedId = id?.trim();
        if (!trimmedId || !UUID_REGEX.test(trimmedId)) {
            throw new ValidationError("Invalid collateral ID format");
        }

        const collateral = await prisma.collateral.findUnique({
            where: { id: trimmedId },
        });

        if (!collateral) {
            throw new NotFoundError("Collateral not found");
        }
        return collateral;
    }

    async getCollateralByMetadataHash(hash: string) {
        const trimmedHash = hash?.trim();
        if (!trimmedHash) {
            throw new ValidationError("Metadata hash is required");
        }

        const collateral = await prisma.collateral.findUnique({
            where: { metadataHash: trimmedHash },
        });

        if (!collateral) {
            throw new NotFoundError("Collateral not found");
        }
        return collateral;
    }

    async listCollateral(query: CollateralListQuery) {
        const page = coercePage(query.page);
        const limit = coerceLimit(query.limit);
        const skip = (page - 1) * limit;

        const where: Record<string, string> = {};
        if (query.escrowId?.trim()) where.escrowId = query.escrowId.trim();
        if (query.status?.trim()) where.status = normalizeStatus(query.status, "status");

        const [items, total] = await Promise.all([
            prisma.collateral.findMany({
                where,
                orderBy: { createdAt: "desc" },
                skip,
                take: limit,
            }),
            prisma.collateral.count({ where }),
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

    startIndexer() {
        if (this.indexerTimer) {
            return;
        }

        console.log("Starting collateral indexer...");
        
        // Background polling of Soroban RPC for CollateralDeposited events
        this.indexerTimer = setInterval(async () => {
            if (this.polling) return;
            this.polling = true;

            try {
                // TODO: Implement actual RPC querying logic for `CollateralDeposited` events
                // Currently simulated:
                // const events = await rpc.getEvents({ ... });
                // const hashes = events.map(e => extractHash(e));
                
                const pendingCollaterals = await prisma.collateral.findMany({
                    where: { status: "PENDING" },
                    select: { id: true, metadataHash: true },
                });

                if (pendingCollaterals.length === 0) {
                    return;
                }

                // If on-chain indexed events match, update to DEPOSITED.
                // E.g.
                // await prisma.collateral.update({ where: { id: ... }, data: { status: "DEPOSITED" } })

            } catch (error) {
                console.error("Collateral indexer polling failed:", error);
            } finally {
                this.polling = false;
            }
        }, 30_000); // run every 30s

        this.indexerTimer.unref();
    }

    stopIndexer() {
        if (this.indexerTimer) {
            clearInterval(this.indexerTimer);
            this.indexerTimer = null;
            this.polling = false;
            console.log("Collateral indexer stopped.");
        }
    }
}

export default new CollateralService();

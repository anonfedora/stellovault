import { EventEmitter } from "events";
import { Keypair } from "@stellar/stellar-sdk";
import {
    ConflictError,
    NotFoundError,
    UnauthorizedError,
    ValidationError,
    TooManyRequestsError,
} from "../config/errors";
import { Prisma, PrismaClient } from "@prisma/client";
import { prisma } from "./database.service";

const CONFIRMATION_RATE_LIMIT_MS = 60 * 1000;
const MAX_CONFIRMATIONS_PER_MINUTE = 10;

type TransactionClient = Omit<PrismaClient, "$connect" | "$disconnect" | "$on" | "$transaction" | "$use" | "$extends">;

type CreateOracleInput = {
    address: string;
};

type ConfirmEventInput = {
    oracleAddress: string;
    escrowId: string;
    eventType: string;
    signature: string;
    payload: Record<string, unknown>;
    nonce: string;
};

function toJsonValue(value: Record<string, unknown>): Prisma.InputJsonValue {
    return value as Prisma.InputJsonValue;
}

type DisputeInput = {
    escrowId: string;
    reason: string;
    disputerAddress: string;
};

type ListOraclesQuery = {
    isActive?: boolean;
    limit?: number;
    offset?: number;
};

type ListConfirmationsQuery = {
    escrowId: string;
    limit?: number;
    offset?: number;
};

function sortObjectKeys(obj: unknown): unknown {
    if (obj === null || typeof obj !== "object") {
        return obj;
    }
    if (Array.isArray(obj)) {
        return obj.map(sortObjectKeys);
    }
    const sorted: Record<string, unknown> = {};
    const keys = Object.keys(obj).sort();
    for (const key of keys) {
        sorted[key] = sortObjectKeys((obj as Record<string, unknown>)[key]);
    }
    return sorted;
}

function canonicalStringify(data: unknown): string {
    return JSON.stringify(sortObjectKeys(data));
}

export const oracleEventEmitter = new EventEmitter();

export class OracleService {
    private requireNonEmptyString(value: unknown, fieldName: string): string {
        if (typeof value !== "string" || value.trim().length === 0) {
            throw new ValidationError(`${fieldName} is required`);
        }
        return value.trim();
    }

    private normalizeAddress(address: unknown): string {
        return this.requireNonEmptyString(address, "address").toUpperCase();
    }

    private assertValidStellarAddress(address: string): void {
        try {
            Keypair.fromPublicKey(address);
        } catch {
            throw new ValidationError("Invalid Stellar wallet address");
        }
    }

    /**
     * Decodes a 64-byte Ed25519 signature from hex or base64 encoding.
     *
     * Accepted formats:
     * - Hex: 128 characters (0-9a-fA-F), decodes to 64 bytes
     * - Base64: Standard base64 with optional = padding, decodes to 64 bytes
     *
     * @param signature - The encoded signature string
     * @returns Buffer containing the 64-byte raw signature
     * @throws ValidationError if the signature is empty, malformed, or not 64 bytes
     *
     * @example
     * // Hex format (128 chars)
     * decodeSignature("a1b2c3...") // 128 hex characters
     *
     * // Base64 format
     * decodeSignature("abc123+==") // base64 string
     */
    private decodeSignature(signature: unknown): Buffer {
        const value = this.requireNonEmptyString(signature, "signature");

        if (/^[0-9a-fA-F]+$/.test(value) && value.length % 2 === 0) {
            const decoded = Buffer.from(value, "hex");
            if (decoded.length === 64) {
                return decoded;
            }
        }

        if (/^[A-Za-z0-9+/]+={0,2}$/.test(value)) {
            const decoded = Buffer.from(value, "base64");
            if (decoded.length === 64) {
                return decoded;
            }
        }

        throw new ValidationError("Signature must be a 64-byte hex or base64 value");
    }

    private getMinuteWindow(): Date {
        const now = new Date();
        return new Date(
            now.getFullYear(),
            now.getMonth(),
            now.getDate(),
            now.getHours(),
            now.getMinutes()
        );
    }

    private async checkRateLimitAtomic(oracleId: string, tx: TransactionClient): Promise<void> {
        const minuteWindow = this.getMinuteWindow();

        const existing = await tx.oracleRateLimit.findUnique({
            where: {
                oracleId_minuteWindow: {
                    oracleId,
                    minuteWindow,
                },
            },
        });

        if (existing && existing.count >= MAX_CONFIRMATIONS_PER_MINUTE) {
            throw new TooManyRequestsError("Rate limit exceeded for oracle confirmations");
        }
    }

    private async incrementRateLimit(oracleId: string, tx: TransactionClient): Promise<void> {
        const minuteWindow = this.getMinuteWindow();

        await tx.oracleRateLimit.upsert({
            where: {
                oracleId_minuteWindow: {
                    oracleId,
                    minuteWindow,
                },
            },
            update: {
                count: { increment: 1 },
            },
            create: {
                oracleId,
                minuteWindow,
                count: 1,
            },
        });
    }

    async registerOracle(input: CreateOracleInput) {
        const address = this.normalizeAddress(input.address);
        this.assertValidStellarAddress(address);

        const existing = await prisma.oracle.findUnique({
            where: { address },
        });

        if (existing) {
            if (existing.isActive) {
                throw new ConflictError("Oracle already registered and active");
            }
            return prisma.oracle.update({
                where: { address },
                data: { isActive: true, deactivatedAt: null },
            });
        }

        try {
            return await prisma.oracle.create({
                data: { address },
            });
        } catch (error) {
            if (
                typeof error === "object" &&
                error !== null &&
                "code" in error &&
                (error as { code?: string }).code === "P2002"
            ) {
                throw new ConflictError("Oracle already registered and active");
            }
            throw error;
        }
    }

    async deactivateOracle(address: string) {
        const normalizedAddress = this.normalizeAddress(address);
        this.assertValidStellarAddress(normalizedAddress);

        const oracle = await prisma.oracle.findUnique({
            where: { address: normalizedAddress },
        });

        if (!oracle) {
            throw new NotFoundError("Oracle not found");
        }

        if (!oracle.isActive) {
            throw new ValidationError("Oracle is already deactivated");
        }

        return prisma.oracle.update({
            where: { address: normalizedAddress },
            data: { isActive: false, deactivatedAt: new Date() },
        });
    }

    async confirmOracleEvent(input: ConfirmEventInput) {
        const oracleAddress = this.normalizeAddress(input.oracleAddress);
        const escrowId = this.requireNonEmptyString(input.escrowId, "escrowId");
        const eventType = this.requireNonEmptyString(input.eventType, "eventType");
        const nonce = this.requireNonEmptyString(input.nonce, "nonce");

        this.assertValidStellarAddress(oracleAddress);

        const oracle = await prisma.oracle.findUnique({
            where: { address: oracleAddress },
        });

        if (!oracle) {
            throw new UnauthorizedError("Unknown oracle address");
        }

        if (!oracle.isActive) {
            throw new UnauthorizedError("Oracle is not active");
        }

        const escrow = await prisma.escrow.findUnique({
            where: { id: escrowId },
        });

        if (!escrow) {
            throw new NotFoundError("Escrow not found");
        }

        const existingConfirmation = await prisma.oracleConfirmation.findUnique({
            where: {
                oracleId_escrowId_eventType: {
                    oracleId: oracle.id,
                    escrowId,
                    eventType,
                },
            },
        });

        if (existingConfirmation) {
            throw new ConflictError("Duplicate confirmation for this event");
        }

        const message = Buffer.from(
            canonicalStringify({
                escrowId,
                eventType,
                nonce,
                payload: input.payload,
            })
        );
        const signatureBytes = this.decodeSignature(input.signature);
        const isValid = Keypair.fromPublicKey(oracleAddress).verify(message, signatureBytes);

        if (!isValid) {
            throw new UnauthorizedError("Invalid oracle signature");
        }

        return prisma.$transaction(async (tx) => {
            await this.checkRateLimitAtomic(oracle.id, tx);
            await this.incrementRateLimit(oracle.id, tx);

            return tx.oracleConfirmation.create({
                data: {
                    oracleId: oracle.id,
                    escrowId,
                    eventType,
                    signature: input.signature,
                    payload: toJsonValue(input.payload),
                },
            });
        });
    }

    async listOracles(query: ListOraclesQuery) {
        const { isActive, limit = 50, offset = 0 } = query;

        const where: Record<string, unknown> = {};
        if (isActive !== undefined) {
            where.isActive = isActive;
        }

        const [oracles, total] = await Promise.all([
            prisma.oracle.findMany({
                where,
                orderBy: { registeredAt: "desc" },
                take: limit,
                skip: offset,
            }),
            prisma.oracle.count({ where }),
        ]);

        return { oracles, total, limit, offset };
    }

    async getOracle(address: string) {
        const normalizedAddress = this.normalizeAddress(address);
        this.assertValidStellarAddress(normalizedAddress);

        const oracle = await prisma.oracle.findUnique({
            where: { address: normalizedAddress },
            include: {
                _count: {
                    select: { confirmations: true },
                },
            },
        });

        if (!oracle) {
            throw new NotFoundError("Oracle not found");
        }

        return oracle;
    }

    async getConfirmations(query: ListConfirmationsQuery) {
        const { escrowId, limit = 50, offset = 0 } = query;

        const escrow = await prisma.escrow.findUnique({
            where: { id: escrowId },
        });

        if (!escrow) {
            throw new NotFoundError("Escrow not found");
        }

        const [confirmations, total] = await Promise.all([
            prisma.oracleConfirmation.findMany({
                where: { escrowId },
                include: {
                    oracle: {
                        select: { address: true, isActive: true },
                    },
                },
                orderBy: { createdAt: "desc" },
                take: limit,
                skip: offset,
            }),
            prisma.oracleConfirmation.count({ where: { escrowId } }),
        ]);

        return { confirmations, total, limit, offset };
    }

    async flagDispute(input: DisputeInput) {
        const { escrowId, reason, disputerAddress } = input;

        const normalizedDisputer = this.normalizeAddress(disputerAddress);
        this.assertValidStellarAddress(normalizedDisputer);

        const disputeReason = this.requireNonEmptyString(reason, "reason");

        const escrow = await prisma.escrow.findUnique({
            where: { id: escrowId },
        });

        if (!escrow) {
            throw new NotFoundError("Escrow not found");
        }

        if (escrow.status === "DISPUTED") {
            throw new ConflictError("Escrow is already disputed");
        }

        const { updatedEscrow, dispute } = await prisma.$transaction(async (tx) => {
            const updateResult = await tx.escrow.updateMany({
                where: {
                    id: escrowId,
                    status: { not: "DISPUTED" },
                },
                data: { status: "DISPUTED" },
            });

            if (updateResult.count === 0) {
                throw new ConflictError("Escrow is already disputed");
            }

            const newDispute = await tx.dispute.create({
                data: {
                    escrowId,
                    reporterAddress: normalizedDisputer,
                    reason: disputeReason,
                },
            });

            const fetchedEscrow = await tx.escrow.findUnique({
                where: { id: escrowId },
            });

            return { updatedEscrow: fetchedEscrow, dispute: newDispute };
        });

        const disputeEvent = {
            escrow: updatedEscrow,
            dispute: {
                id: dispute.id,
                escrowId,
                reason: disputeReason,
                disputerAddress: normalizedDisputer,
                createdAt: dispute.createdAt,
            },
        };

        oracleEventEmitter.emit("dispute", disputeEvent);

        return disputeEvent;
    }

    async getOracleMetrics() {
        const thirtyDaysAgo = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);

        const [
            activeOracles,
            totalOracles,
            totalConfirmations,
            escrowsByStatus,
            recentConfirmations,
            confirmationsPerOracle,
        ] = await Promise.all([
            prisma.oracle.count({ where: { isActive: true } }),
            prisma.oracle.count(),
            prisma.oracleConfirmation.count(),
            prisma.escrow.groupBy({
                by: ["status"],
                _count: { id: true },
            }),
            prisma.oracleConfirmation.count({
                where: { createdAt: { gte: thirtyDaysAgo } },
            }),
            prisma.oracleConfirmation.groupBy({
                by: ["oracleId"],
                _count: { id: true },
            }),
        ]);

        const avgConfirmationsPerOracle =
            totalOracles > 0
                ? confirmationsPerOracle.reduce((sum, o) => sum + o._count.id, 0) / totalOracles
                : 0;

        const escrowCounts: Record<string, number> = {};
        escrowsByStatus.forEach((item) => {
            escrowCounts[item.status] = item._count.id;
        });

        return {
            activeOracles,
            totalOracles,
            inactiveOracles: totalOracles - activeOracles,
            totalConfirmations,
            confirmationsLast30Days: recentConfirmations,
            avgConfirmationsPerOracle: Math.round(avgConfirmationsPerOracle * 100) / 100,
            escrowCounts,
        };
    }
}

export default new OracleService();

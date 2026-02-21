import { randomBytes } from "crypto";
import { Keypair } from "@stellar/stellar-sdk";
import {
    ConflictError,
    NotFoundError,
    UnauthorizedError,
    ValidationError,
} from "../config/errors";
import { prisma } from "./database.service";

const CHALLENGE_TTL_MS = 10 * 60 * 1000;
const CHALLENGE_PURPOSE = {
    LOGIN: "LOGIN",
    LINK_WALLET: "LINK_WALLET",
} as const;
type ChallengePurpose = (typeof CHALLENGE_PURPOSE)[keyof typeof CHALLENGE_PURPOSE];

type DbClient = any;

export class AuthService {
    private requireNonEmptyString(value: unknown, fieldName: string): string {
        if (typeof value !== "string" || value.trim().length === 0) {
            throw new ValidationError(`${fieldName} is required`);
        }
        return value.trim();
    }

    private normalizeAddress(address: unknown): string {
        return this.requireNonEmptyString(address, "walletAddress").toUpperCase();
    }

    private assertValidStellarAddress(address: string): void {
        try {
            Keypair.fromPublicKey(address);
        } catch {
            throw new ValidationError("Invalid Stellar wallet address");
        }
    }

    private buildChallengeMessage(purpose: ChallengePurpose, nonce: string): string {
        const action = purpose === CHALLENGE_PURPOSE.LOGIN ? "login" : "link-wallet";
        return `stellovault:${action}:${nonce}`;
    }

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

    private async verifyAndConsumeChallenge(
        db: DbClient,
        address: string,
        nonce: string,
        signature: string,
        purpose: ChallengePurpose,
        userId?: string
    ): Promise<void> {
        const normalizedNonce = this.requireNonEmptyString(nonce, "nonce");
        const now = new Date();

        const message = Buffer.from(this.buildChallengeMessage(purpose, normalizedNonce));
        const signatureBytes = this.decodeSignature(signature);
        const isValid = Keypair.fromPublicKey(address).verify(message, signatureBytes);
        if (!isValid) {
            throw new UnauthorizedError("Invalid signature");
        }

        const where: Record<string, unknown> = {
            walletAddress: address,
            nonce: normalizedNonce,
            purpose,
            usedAt: null,
            expiresAt: { gt: now },
        };
        if (userId) {
            where.userId = userId;
        }

        // Atomic consume prevents concurrent double-use of the same challenge.
        const consumeResult = await db.walletChallenge.updateMany({
            where,
            data: { usedAt: now },
        });
        if (consumeResult.count !== 1) {
            throw new UnauthorizedError("Invalid or expired challenge");
        }
    }

    private async lockUserRow(tx: DbClient, userId: string): Promise<void> {
        const rows = await tx.$queryRaw`SELECT id FROM "User" WHERE id = ${userId} FOR UPDATE`;
        if (!Array.isArray(rows) || rows.length === 0) {
            throw new NotFoundError("User not found");
        }
    }

    async generateChallenge(
        address: string,
        purpose: ChallengePurpose = CHALLENGE_PURPOSE.LOGIN,
        userId?: string
    ): Promise<{ nonce: string; expiresAt: Date; message: string }> {
        const normalizedAddress = this.normalizeAddress(address);
        this.assertValidStellarAddress(normalizedAddress);

        if (purpose === CHALLENGE_PURPOSE.LINK_WALLET && !userId) {
            throw new ValidationError("userId is required for wallet linking challenges");
        }

        const nonce = randomBytes(24).toString("hex");
        const expiresAt = new Date(Date.now() + CHALLENGE_TTL_MS);
        const db = prisma;

        await db.walletChallenge.create({
            data: {
                walletAddress: normalizedAddress,
                nonce,
                purpose,
                userId: userId ?? null,
                expiresAt,
            },
        });

        return {
            nonce,
            expiresAt,
            message: this.buildChallengeMessage(purpose, nonce),
        };
    }

    async getUserWallets(userId: string) {
        const db = prisma;
        return db.wallet.findMany({
            where: { userId },
            orderBy: [{ isPrimary: "desc" }, { createdAt: "asc" }],
        });
    }

    async linkWallet(
        userId: string,
        address: string,
        nonce: string,
        signature: string,
        label?: string
    ) {
        const normalizedAddress = this.normalizeAddress(address);
        this.assertValidStellarAddress(normalizedAddress);

        try {
            return await prisma.$transaction(async (tx: any) => {
                await this.lockUserRow(tx, userId);

                const existingWallet = await tx.wallet.findUnique({
                    where: { address: normalizedAddress },
                    select: { id: true },
                });
                if (existingWallet) {
                    throw new ConflictError("Wallet address is already linked");
                }

                const addressInUseByUser = await tx.user.findUnique({
                    where: { stellarAddress: normalizedAddress },
                    select: { id: true },
                });
                if (addressInUseByUser && addressInUseByUser.id !== userId) {
                    throw new ConflictError("Wallet address is already linked");
                }

                await this.verifyAndConsumeChallenge(
                    tx,
                    normalizedAddress,
                    nonce,
                    signature,
                    CHALLENGE_PURPOSE.LINK_WALLET,
                    userId
                );

                const walletCount = await tx.wallet.count({ where: { userId } });
                const isPrimary = walletCount === 0;

                const createdWallet = await tx.wallet.create({
                    data: {
                        userId,
                        address: normalizedAddress,
                        label: label?.trim() || null,
                        isPrimary,
                    },
                });

                if (isPrimary) {
                    await tx.user.update({
                        where: { id: userId },
                        data: { stellarAddress: normalizedAddress },
                    });
                }

                return createdWallet;
            });
        } catch (error) {
            if (
                typeof error === "object" &&
                error !== null &&
                "code" in error &&
                (error as { code?: string }).code === "P2002"
            ) {
                throw new ConflictError("Wallet address is already linked");
            }
            throw error;
        }
    }

    async unlinkWallet(userId: string, walletId: string): Promise<void> {
        await prisma.$transaction(async (tx: any) => {
            await this.lockUserRow(tx, userId);

            const wallets = await tx.wallet.findMany({
                where: { userId },
                orderBy: { createdAt: "asc" },
            });

            const wallet = wallets.find((item: { id: string }) => item.id === walletId);
            if (!wallet) {
                throw new NotFoundError("Wallet not found");
            }

            if (wallets.length <= 1) {
                throw new ValidationError("Cannot unlink the only wallet");
            }

            if (wallet.isPrimary) {
                const replacement = wallets.find((item: { id: string }) => item.id !== walletId);
                if (!replacement) {
                    throw new ValidationError("Cannot unlink the only primary wallet");
                }

                await tx.wallet.updateMany({
                    where: { userId },
                    data: { isPrimary: false },
                });
                await tx.wallet.update({
                    where: { id: replacement.id },
                    data: { isPrimary: true },
                });
                await tx.user.update({
                    where: { id: userId },
                    data: { stellarAddress: replacement.address },
                });
            }

            await tx.wallet.delete({ where: { id: walletId } });
        });
    }

    async setPrimaryWallet(userId: string, walletId: string) {
        return prisma.$transaction(async (tx: any) => {
            await this.lockUserRow(tx, userId);

            const wallet = await tx.wallet.findFirst({
                where: { id: walletId, userId },
            });
            if (!wallet) {
                throw new NotFoundError("Wallet not found");
            }

            await tx.wallet.updateMany({
                where: { userId },
                data: { isPrimary: false },
            });

            const updatedWallet = await tx.wallet.update({
                where: { id: walletId },
                data: { isPrimary: true },
            });

            await tx.user.update({
                where: { id: userId },
                data: { stellarAddress: updatedWallet.address },
            });

            return updatedWallet;
        });
    }

    async updateWalletLabel(userId: string, walletId: string, label?: string) {
        const db = prisma;
        const wallet = await db.wallet.findFirst({ where: { id: walletId, userId } });
        if (!wallet) {
            throw new NotFoundError("Wallet not found");
        }

        return db.wallet.update({
            where: { id: walletId },
            data: { label: label?.trim() || null },
        });
    }
}

export default new AuthService();

import { prisma } from "../config/prisma";
import jwt, { SignOptions } from "jsonwebtoken";
import { Keypair } from "@stellar/stellar-sdk";
import { randomBytes, randomUUID } from "crypto";
import { env } from "../config/env";
import { JWT_CONFIG, TokenPayload } from "../config/jwt";
import { UnauthorizedError, ValidationError, NotFoundError, ConflictError } from "../config/errors";
import { UserRole, PermissionScope } from "../types/rbac";

const CHALLENGE_TTL_MS = 5 * 60 * 1000; // 5 minutes

const CHALLENGE_PURPOSE = {
    LOGIN: "LOGIN",
    LINK_WALLET: "LINK_WALLET",
} as const;
type ChallengePurpose = (typeof CHALLENGE_PURPOSE)[keyof typeof CHALLENGE_PURPOSE];

/**
 * Validate if a string is a valid Stellar public key (starting with 'G' and 56 chars)
 */
function isValidStellarAddress(address: string): boolean {
    try {
        Keypair.fromPublicKey(address);
        return true;
    } catch {
        return false;
    }
}

type DbClient = typeof prisma | Parameters<Parameters<typeof prisma.$transaction>[0]>[0];

export class AuthService {
    // --- PRIVATE HELPERS ---

    private normalizeAddress(address: string): string {
        if (!address || address.trim().length === 0) {
            throw new ValidationError("walletAddress is required");
        }
        return address.trim().toUpperCase();
    }

    private assertValidStellarAddress(address: string): void {
        if (!isValidStellarAddress(address)) {
            throw new ValidationError("Invalid Stellar wallet address");
        }
    }

    private async lockUserRow(tx: DbClient, userId: string): Promise<void> {
        const rows = await (tx as any).$queryRaw`SELECT id FROM "User" WHERE id = ${userId} FOR UPDATE`;
        if (!Array.isArray(rows) || rows.length === 0) {
            throw new NotFoundError("User not found");
        }
    }

    private buildChallengeMessage(purpose: ChallengePurpose, nonce: string): string {
        const action = purpose === CHALLENGE_PURPOSE.LOGIN ? "login" : "link-wallet";
        return `stellovault:${action}:${nonce}`;
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

        // Upsert user/wallet for login flow
        const wallet = await prisma.wallet.upsert({
            where: { address: normalizedAddress },
            update: {},
            create: {
                user: { create: { stellarAddress: normalizedAddress, name: null } },
                address: normalizedAddress,
                isPrimary: true,
            },
            include: { user: true },
        });

        await prisma.walletChallenge.create({
            data: {
                userId: userId ?? wallet.userId,
                walletAddress: normalizedAddress,
                nonce,
                expiresAt,
                purpose,
            },
        });

        const message = this.buildChallengeMessage(purpose, nonce);
        return { nonce, expiresAt, message };
    }

    async verifySignature(walletAddress: string, nonce: string, signature: string, ipAddress?: string, userAgent?: string) {
        const normalizedAddress = this.normalizeAddress(walletAddress);
        this.assertValidStellarAddress(normalizedAddress);

        const challenge = await prisma.walletChallenge.findFirst({
            where: {
                nonce,
                consumed: false,
                expiresAt: { gt: new Date() },
                walletAddress: normalizedAddress,
            },
        });

        if (!challenge) {
            throw new UnauthorizedError("Invalid or expired challenge");
        }

        const messageToSign = this.buildChallengeMessage(CHALLENGE_PURPOSE.LOGIN, nonce);

        try {
            const keypair = Keypair.fromPublicKey(normalizedAddress);
            const isValid = keypair.verify(
                Buffer.from(messageToSign, "utf-8"),
                Buffer.from(signature, "base64")
            );

            if (!isValid) {
                throw new UnauthorizedError("Signature verification failed");
            }
        } catch (err) {
            if (err instanceof UnauthorizedError) throw err;
            throw new UnauthorizedError("Signature verification failed");
        }

        await prisma.walletChallenge.update({
            where: { id: challenge.id },
            data: { consumed: true },
        });

        const user = await prisma.user.findUnique({
            where: { id: challenge.userId },
        });

        if (!user) {
            throw new NotFoundError("User not found for this wallet");
        }

        const jti = randomUUID();
        const accessTokenExpiresIn = env.jwt.accessExpiresIn || JWT_CONFIG.ACCESS_TOKEN_EXPIRY;
        const refreshTokenExpiresIn = env.jwt.refreshExpiresIn || JWT_CONFIG.REFRESH_TOKEN_EXPIRY;
        const refreshExpirySeconds = this.parseExpiry(refreshTokenExpiresIn);

        await prisma.session.create({
            data: {
                userId: user.id,
                jti,
                revokedAt: null,
            },
        });

        const tokenPayload: TokenPayload = {
            userId: user.id,
            jti,
            walletAddress: normalizedAddress,
            role: user.role as UserRole,
        };

        return {
            accessToken: jwt.sign(tokenPayload, env.jwt.accessSecret, { expiresIn: accessTokenExpiresIn } as SignOptions),
            refreshToken: jwt.sign(tokenPayload, env.jwt.refreshSecret, { expiresIn: refreshTokenExpiresIn } as SignOptions),
            user: {
                id: user.id,
                name: user.name,
                role: user.role,
                stellarAddress: normalizedAddress,
            },
        };
    }

    async refreshTokens(refreshToken: string) {
        let payload: TokenPayload;
        try {
            payload = jwt.verify(refreshToken, env.jwt.refreshSecret) as TokenPayload;
        } catch {
            throw new UnauthorizedError("Invalid or expired refresh token");
        }

        const session = await prisma.session.findUnique({
            where: { jti: payload.jti },
        });

        if (!session || session.revokedAt !== null) {
            throw new UnauthorizedError("Session revoked or expired");
        }

        const user = await prisma.user.findUnique({
            where: { id: session.userId },
        });

        if (!user) {
            throw new NotFoundError("User not found");
        }

        const accessTokenExpiresIn = env.jwt.accessExpiresIn || JWT_CONFIG.ACCESS_TOKEN_EXPIRY;
        const newJti = randomUUID();

        const tokenPayload: TokenPayload = {
            userId: user.id,
            jti: newJti,
            walletAddress: payload.walletAddress,
            role: user.role as UserRole,
        };

        const newAccessToken = jwt.sign(tokenPayload, env.jwt.accessSecret, { expiresIn: accessTokenExpiresIn } as SignOptions);
        const newRefreshToken = jwt.sign(tokenPayload, env.jwt.refreshSecret, { expiresIn: JWT_CONFIG.REFRESH_TOKEN_EXPIRY } as SignOptions);

        await prisma.$transaction(async (tx) => {
            await tx.session.update({
                where: { jti: session.jti },
                data: { revokedAt: new Date() },
            });
            await tx.session.create({
                data: {
                    jti: newJti,
                    userId: user.id,
                    revokedAt: null,
                },
            });
        });

        return {
            accessToken: newAccessToken,
            refreshToken: newRefreshToken,
            user: {
                id: user.id,
                name: user.name,
                role: user.role,
                stellarAddress: payload.walletAddress,
            },
        };
    }

    async revokeSession(jti: string) {
        const session = await prisma.session.findUnique({ where: { jti } });
        if (!session) throw new NotFoundError("Session not found");

        await prisma.session.update({
            where: { jti },
            data: { revokedAt: new Date() },
        });
    }

    async revokeAllSessions(userId: string): Promise<number> {
        const result = await prisma.session.updateMany({
            where: { userId, revokedAt: null },
            data: { revokedAt: new Date() },
        });
        return result.count;
    }

    async getUserById(userId: string) {
        const user = await prisma.user.findUnique({
            where: { id: userId },
            select: {
                id: true,
                name: true,
                role: true,
                createdAt: true,
                updatedAt: true,
                wallets: {
                    select: {
                        id: true,
                        address: true,
                        isPrimary: true,
                        label: true,
                        createdAt: true,
                        updatedAt: true,
                    },
                },
            },
        });
        if (!user) throw new NotFoundError("User not found");
        return user;
    }

    async getUserWallets(userId: string) {
        const db = prisma;
        return db.wallet.findMany({
            where: { userId },
            orderBy: [{ isPrimary: "desc" }, { createdAt: "asc" }],
        });
    }

    // --- WALLET MANAGEMENT ---

    async linkWallet(userId: string, address: string, nonce: string, signature: string, label?: string) {
        const normalizedAddress = this.normalizeAddress(address);
        this.assertValidStellarAddress(normalizedAddress);

        return await prisma.$transaction(async (tx) => {
            await this.lockUserRow(tx, userId);

            const existingWallet = await tx.wallet.findUnique({
                where: { address: normalizedAddress },
            });
            if (existingWallet) {
                throw new ConflictError("Wallet address is already linked");
            }

            const challenge = await tx.walletChallenge.findFirst({
                where: {
                    nonce,
                    consumed: false,
                    expiresAt: { gt: new Date() },
                    userId,
                },
            });

            if (!challenge) throw new UnauthorizedError("Invalid or expired linking challenge");

            const messageToSign = `stellovault:link-wallet:${nonce}`;
            const keypair = Keypair.fromPublicKey(normalizedAddress);
            const isValid = keypair.verify(Buffer.from(messageToSign, "utf-8"), Buffer.from(signature, "base64"));

            if (!isValid) throw new UnauthorizedError("Invalid signature");

            await tx.walletChallenge.update({ where: { id: challenge.id }, data: { consumed: true } });

            const walletCount = await tx.wallet.count({ where: { userId } });
            const isPrimary = walletCount === 0;

            return await tx.wallet.create({
                data: {
                    userId,
                    address: normalizedAddress,
                    label: label?.trim() || null,
                    isPrimary,
                },
            });
        });
    }

    async unlinkWallet(userId: string, walletId: string): Promise<void> {
        await prisma.$transaction(async (tx) => {
            await this.lockUserRow(tx, userId);

            const wallets = await tx.wallet.findMany({
                where: { userId },
                orderBy: { createdAt: "asc" },
            });

            const wallet = wallets.find((item) => item.id === walletId);
            if (!wallet) throw new NotFoundError("Wallet not found");
            if (wallets.length <= 1) throw new ValidationError("Cannot unlink the only wallet");

            if (wallet.isPrimary) {
                const replacement = wallets.find((item) => item.id !== walletId);
                if (!replacement) {
                    throw new ValidationError("No backup wallet found to promote to primary");
                }
                await tx.wallet.update({
                    where: { id: replacement.id },
                    data: { isPrimary: true },
                });
            }

            await tx.wallet.delete({ where: { id: walletId } });
        });
    }

    async setPrimaryWallet(userId: string, walletId: string) {
        return prisma.$transaction(async (tx) => {
            await this.lockUserRow(tx, userId);

            const wallet = await tx.wallet.findFirst({
                where: { id: walletId, userId },
            });
            if (!wallet) throw new NotFoundError("Wallet not found");

            await tx.wallet.updateMany({
                where: { userId },
                data: { isPrimary: false },
            });

            return await tx.wallet.update({
                where: { id: walletId },
                data: { isPrimary: true },
            });
        });
    }

    async updateWalletLabel(userId: string, walletId: string, label?: string) {
        const db = prisma;
        const wallet = await db.wallet.findFirst({ where: { id: walletId, userId } });
        if (!wallet) {
            throw new NotFoundError("Wallet not found");
        }

        return prisma.wallet.update({
            where: { id: walletId },
            data: { label: label?.trim() || null },
        });
    }

    private parseExpiry(expiryStr: string): number {
        const match = expiryStr.match(/^(\d+)([smhd])$/);
        if (!match) {
            console.warn(`Unrecognized expiry format "${expiryStr}", defaulting to 900s`);
            return 900;
        }
        const value = parseInt(match[1], 10);
        const unit = match[2];
        switch (unit) {
            case "s": return value;
            case "m": return value * 60;
            case "h": return value * 3600;
            case "d": return value * 86400;
            default: return 900;
        }
    }
}

export const authService = new AuthService();

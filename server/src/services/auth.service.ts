import { prisma } from "../config/prisma";
import jwt, { SignOptions } from "jsonwebtoken";
import { Keypair } from "@stellar/stellar-sdk";
import { v4 as uuidv4 } from "uuid";
import { env } from "../config/env";
import { JWT_CONFIG, TokenPayload } from "../config/jwt";
import { UnauthorizedError, ValidationError, NotFoundError, ConflictError } from "../config/errors";

const CHALLENGE_MESSAGE_PREFIX = "stellovault:login:";
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

export class AuthService {
    // --- PRIVATE HELPERS ---

    private normalizeAddress(address: string): string {
        if (!address || address.trim().length === 0) {
            throw new ValidationError("walletAddress is required");
        }
        return address.trim().toUpperCase();
    }

    private async lockUserRow(tx: any, userId: string): Promise<void> {
        const rows = await tx.$queryRaw`SELECT id FROM "User" WHERE id = ${userId} FOR UPDATE`;
        if (!Array.isArray(rows) || rows.length === 0) {
            throw new NotFoundError("User not found");
        }
    }

    private buildChallengeMessage(purpose: ChallengePurpose, nonce: string): string {
        const action = purpose === CHALLENGE_PURPOSE.LOGIN ? "login" : "link-wallet";
        return `stellovault:${action}:${nonce}`;
    }

    // --- ORIGINAL FUNCTIONS (KEPT AS IS) ---

    async generateChallenge(
        walletAddress: string,
        purpose: ChallengePurpose = "LOGIN", // Default to LOGIN
        userId?: string                       // Optional, required for LINK_WALLET
    ) {
        if (!isValidStellarAddress(walletAddress)) {
            throw new ValidationError("Invalid Stellar wallet address");
        }

        const nonce = uuidv4();
        const expiresAt = new Date(Date.now() + JWT_CONFIG.CHALLENGE_EXPIRY_SECONDS * 1000);

        let wallet = await prisma.wallet.findUnique({
            where: { stellarAddress: walletAddress },
            include: { user: true },
        });

        if (!wallet) {
            const user = await prisma.user.create({
                data: { name: null },
            });

            wallet = await prisma.wallet.create({
                data: {
                    userId: user.id,
                    stellarAddress: walletAddress,
                    isPrimary: true,
                    status: "ACTIVE",
                },
                include: { user: true },
            });
        }

        const challenge = await prisma.challenge.create({
            data: {
                userId: wallet.userId,
                walletId: wallet.id,
                nonce,
                expiresAt,
            },
        });

        return { nonce: challenge.nonce, expiresAt: challenge.expiresAt };
    }

    async verifySignature(walletAddress: string, nonce: string, signature: string, ipAddress?: string, userAgent?: string) {
        if (!isValidStellarAddress(walletAddress)) {
            throw new ValidationError("Invalid Stellar wallet address");
        }

        const challenge = await prisma.challenge.findFirst({
            where: {
                nonce,
                consumed: false,
                expiresAt: { gt: new Date() },
                wallet: { stellarAddress: walletAddress },
            },
            include: { wallet: true },
        });

        if (!challenge) {
            throw new UnauthorizedError("Invalid or expired challenge");
        }

        const messageToSign = CHALLENGE_MESSAGE_PREFIX + nonce;

        try {
            const keypair = Keypair.fromPublicKey(walletAddress);
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

        await prisma.challenge.update({
            where: { id: challenge.id },
            data: { consumed: true },
        });

        const user = await prisma.user.findUnique({
            where: { id: challenge.userId },
        });

        if (!user) {
            throw new NotFoundError("User not found for this wallet");
        }

        const jti = uuidv4();
        const accessTokenExpiresIn = env.jwt.accessExpiresIn || JWT_CONFIG.ACCESS_TOKEN_EXPIRY;
        const refreshTokenExpiresIn = env.jwt.refreshExpiresIn || JWT_CONFIG.REFRESH_TOKEN_EXPIRY;
        const refreshExpirySeconds = this.parseExpiry(refreshTokenExpiresIn);

        const session = await prisma.session.create({
            data: {
                userId: user.id,
                walletId: challenge.walletId,
                jti,
                ipAddress: ipAddress || null,
                userAgent: userAgent || null,
                expiresAt: new Date(Date.now() + refreshExpirySeconds * 1000),
            },
        });

        const tokenPayload: TokenPayload = {
            userId: user.id,
            jti,
            walletAddress: walletAddress,
        };

        return {
            accessToken: jwt.sign(tokenPayload, env.jwt.accessSecret, { expiresIn: accessTokenExpiresIn } as SignOptions),
            refreshToken: jwt.sign(tokenPayload, env.jwt.refreshSecret, { expiresIn: refreshTokenExpiresIn } as SignOptions),
            user: {
                id: user.id,
                name: user.name,
                role: user.role,
                stellarAddress: walletAddress,
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
            include: { wallet: true },
        });

        if (!session || session.revoked || session.expiresAt < new Date()) {
            throw new UnauthorizedError("Session revoked or expired");
        }

        const user = await prisma.user.findUnique({
            where: { id: session.userId },
        });

        if (!user) {
            throw new NotFoundError("User not found");
        }

        const accessTokenExpiresIn = env.jwt.accessExpiresIn || JWT_CONFIG.ACCESS_TOKEN_EXPIRY;
        const tokenPayload: TokenPayload = {
            userId: user.id,
            jti: session.jti,
            walletAddress: session.wallet.stellarAddress,
        };

        const newAccessToken = jwt.sign(tokenPayload, env.jwt.accessSecret, { expiresIn: accessTokenExpiresIn } as SignOptions);

        return {
            accessToken: newAccessToken,
            refreshToken,
            user: {
                id: user.id,
                name: user.name,
                role: user.role,
                stellarAddress: session.wallet.stellarAddress,
            },
        };
    }

    async revokeSession(jti: string) {
        const session = await prisma.session.findUnique({ where: { jti } });
        if (!session) throw new NotFoundError("Session not found");

        await prisma.session.update({
            where: { jti },
            data: { revoked: true, revokedAt: new Date() },
        });
    }

    async revokeAllSessions(userId: string): Promise<number> {
        const result = await prisma.session.updateMany({
            where: { userId, revoked: false },
            data: { revoked: true, revokedAt: new Date() },
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
                        stellarAddress: true,
                        isPrimary: true,
                        label: true,
                        status: true,
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
        const user = await prisma.user.findUnique({
            where: { id: userId },
            select: {
                wallets: {
                    select: {
                        id: true,
                        stellarAddress: true,
                        isPrimary: true,
                        label: true,
                        status: true,
                        createdAt: true,
                        updatedAt: true,
                    },
                },
            },
        });
        if (!user) throw new NotFoundError("User not found");
        return user.wallets.map(wallet => ({
            id: wallet.id,
            address: wallet.stellarAddress,
            primary: wallet.isPrimary,
            label: wallet.label,
            status: wallet.status,
            createdAt: wallet.createdAt,
            updatedAt: wallet.updatedAt,
        }));
    }

    // --- ADDED NEW FUNCTIONS (FOR WALLET MANAGEMENT) ---

    async linkWallet(userId: string, address: string, nonce: string, signature: string, label?: string) {
        const normalizedAddress = this.normalizeAddress(address);
        if (!isValidStellarAddress(normalizedAddress)) {
            throw new ValidationError("Invalid Stellar wallet address");
        }

        return await prisma.$transaction(async (tx) => {
            await this.lockUserRow(tx, userId);

            const existingWallet = await tx.wallet.findUnique({
                where: { stellarAddress: normalizedAddress },
            });
            if (existingWallet) {
                throw new ConflictError("Wallet address is already linked");
            }

            // Verify the LINK_WALLET challenge
            const challenge = await tx.challenge.findFirst({
                where: {
                    nonce,
                    consumed: false,
                    expiresAt: { gt: new Date() },
                    userId: userId
                }
            });

            if (!challenge) throw new UnauthorizedError("Invalid or expired linking challenge");

            const messageToSign = `stellovault:link-wallet:${nonce}`;
            const keypair = Keypair.fromPublicKey(normalizedAddress);
            const isValid = keypair.verify(Buffer.from(messageToSign, "utf-8"), Buffer.from(signature, "base64"));

            if (!isValid) throw new UnauthorizedError("Invalid signature");

            await tx.challenge.update({ where: { id: challenge.id }, data: { consumed: true } });

            const walletCount = await tx.wallet.count({ where: { userId } });
            const isPrimary = walletCount === 0;

            return await tx.wallet.create({
                data: {
                    userId,
                    stellarAddress: normalizedAddress,
                    label: label?.trim() || null,
                    isPrimary,
                    status: "ACTIVE"
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

            const wallet = wallets.find((item: any) => item.id === walletId);
            if (!wallet) throw new NotFoundError("Wallet not found");
            if (wallets.length <= 1) throw new ValidationError("Cannot unlink the only wallet");

            if (wallet.isPrimary) {
                const replacement = wallets.find((item: any) => item.id !== walletId);
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
        const wallet = await prisma.wallet.findFirst({ where: { id: walletId, userId } });
        if (!wallet) throw new NotFoundError("Wallet not found");

        return prisma.wallet.update({
            where: { id: walletId },
            data: { label: label?.trim() || null },
        });
    }

    private parseExpiry(expiryStr: string): number {
        const match = expiryStr.match(/^(\d+)([smhd])$/);
        if (!match) return 900;
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
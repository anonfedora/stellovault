import { Request, Response, NextFunction } from "express";
import { authService } from "../services/auth.service";

/**
 * POST /api/auth/challenge
 * Generate a nonce for wallet-based login.
 */
export async function requestChallenge(req: Request, res: Response, next: NextFunction) {
    try {
        const { walletAddress } = req.body;

        if (!walletAddress) {
            return res.status(400).json({ success: false, error: "walletAddress is required" });
        }

        const challenge = await authService.generateChallenge(walletAddress);
        res.status(200).json({ success: true, data: challenge });
    } catch (err) {
        next(err);
    }
}

/**
 * POST /api/auth/verify
 * Verify signed nonce and return JWT access + refresh tokens.
 */
export async function verifySignature(req: Request, res: Response, next: NextFunction) {
    try {
        const { walletAddress, nonce, signature } = req.body;

        if (!walletAddress || !nonce || !signature) {
            return res.status(400).json({
                success: false,
                error: "walletAddress, nonce, and signature are required",
            });
        }

        const tokens = await authService.verifySignature(
            walletAddress,
            nonce,
            signature,
            req.ip,
            req.get("user-agent")
        );

        res.status(200).json({ success: true, data: tokens });
    } catch (err) {
        next(err);
    }
}

/**
 * POST /api/auth/refresh
 * Rotate access token using a valid refresh token.
 */
export async function refreshToken(req: Request, res: Response, next: NextFunction) {
    try {
        const { refreshToken } = req.body;

        if (!refreshToken) {
            return res.status(400).json({ success: false, error: "refreshToken is required" });
        }

        const tokens = await authService.refreshTokens(refreshToken);
        res.status(200).json({ success: true, data: tokens });
    } catch (err) {
        next(err);
    }
}

/**
 * POST /api/auth/logout
 * Revoke current session.
 */
export async function logout(req: Request, res: Response, next: NextFunction) {
    try {
        const jti = req.user?.jti;
        if (!jti) {
            return res.status(401).json({ success: false, error: "Unauthorized" });
        }

        await authService.revokeSession(jti);
        res.status(204).send();
    } catch (err) {
        next(err);
    }
}

/**
 * POST /api/auth/logout-all
 * Revoke all sessions for the authenticated user.
 */
export async function logoutAll(req: Request, res: Response, next: NextFunction) {
    try {
        const userId = req.user?.userId;
        if (!userId) {
            return res.status(401).json({ success: false, error: "Unauthorized" });
        }

        const revokedCount = await authService.revokeAllSessions(userId);
        res.status(200).json({ success: true, data: { revokedSessions: revokedCount } });
    } catch (err) {
        next(err);
    }
}

/**
 * GET /api/auth/me
 * Return current authenticated user profile.
 */
export async function getMe(req: Request, res: Response, next: NextFunction) {
    try {
        const userId = req.user?.userId;
        if (!userId) {
            return res.status(401).json({ success: false, error: "Unauthorized" });
        }

        const user = await authService.getUserById(userId);
        res.status(200).json({ success: true, data: user });
    } catch (err) {
        next(err);
    }
}

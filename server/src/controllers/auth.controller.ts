import { Request, Response, NextFunction } from "express";

// TODO: Inject AuthService
// import authService from "../services/auth.service";

/**
 * POST /api/auth/challenge
 * Generate a nonce for wallet-based login.
 */
export async function requestChallenge(req: Request, res: Response, next: NextFunction) {
    try {
        const { walletAddress } = req.body;
        // const challenge = await authService.generateChallenge(walletAddress);
        res.json({ success: true, data: { nonce: "TODO", expiresAt: new Date() } });
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
        // const tokens = await authService.verifySignature(walletAddress, nonce, signature, req.ip);
        res.json({ success: true, data: { accessToken: "TODO", refreshToken: "TODO" } });
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
        // const tokens = await authService.refreshTokens(refreshToken);
        res.json({ success: true, data: { accessToken: "TODO", refreshToken: "TODO" } });
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
        // await authService.revokeSession(req.user!.jti);
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
        // const count = await authService.revokeAllSessions(req.user!.userId);
        res.json({ success: true, data: { revokedSessions: 0 } });
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
        // const user = await authService.getUserById(req.user!.userId);
        res.json({ success: true, data: null });
    } catch (err) {
        next(err);
    }
}

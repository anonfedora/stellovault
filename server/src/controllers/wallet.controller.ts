import { Request, Response, NextFunction } from "express";
import { UnauthorizedError } from "../config/errors";
import authService from "../services/auth.service";

function getUserId(req: Request): string {
    const userId = req.user?.userId;
    if (!userId) {
        throw new UnauthorizedError("Unauthorized");
    }
    return userId;
}

/**
 * GET /api/wallets
 */
export async function listWallets(req: Request, res: Response, next: NextFunction) {
    try {
        const wallets = await authService.getUserWallets(getUserId(req));
        res.json({ success: true, data: wallets });
    } catch (err) { next(err); }
}

/**
 * POST /api/wallets/challenge
 */
export async function walletChallenge(req: Request, res: Response, next: NextFunction) {
    try {
        const { walletAddress } = req.body;
        const challenge = await authService.generateChallenge(
            walletAddress,
            "LINK_WALLET",
            getUserId(req)
        );
        res.json({ success: true, data: challenge });
    } catch (err) { next(err); }
}

/**
 * POST /api/wallets
 */
export async function linkWallet(req: Request, res: Response, next: NextFunction) {
    try {
        const { walletAddress, nonce, signature, label } = req.body;
        const wallet = await authService.linkWallet(
            getUserId(req),
            walletAddress,
            nonce,
            signature,
            label
        );
        res.status(201).json({ success: true, data: wallet });
    } catch (err) { next(err); }
}

/**
 * DELETE /api/wallets/:id
 */
export async function unlinkWallet(req: Request, res: Response, next: NextFunction) {
    try {
        await authService.unlinkWallet(getUserId(req), req.params.id);
        res.status(204).send();
    } catch (err) { next(err); }
}

/**
 * PUT /api/wallets/:id/primary
 */
export async function setPrimaryWallet(req: Request, res: Response, next: NextFunction) {
    try {
        const wallet = await authService.setPrimaryWallet(getUserId(req), req.params.id);
        res.json({ success: true, data: wallet });
    } catch (err) { next(err); }
}

/**
 * PATCH /api/wallets/:id
 */
export async function updateWallet(req: Request, res: Response, next: NextFunction) {
    try {
        const { label } = req.body;
        const wallet = await authService.updateWalletLabel(getUserId(req), req.params.id, label);
        res.json({ success: true, data: wallet });
    } catch (err) { next(err); }
}

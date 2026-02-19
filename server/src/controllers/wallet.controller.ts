import { Request, Response, NextFunction } from "express";

// TODO: Inject WalletService / AuthService

/**
 * GET /api/wallets
 */
export async function listWallets(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: [] });
    } catch (err) { next(err); }
}

/**
 * POST /api/wallets/challenge
 */
export async function walletChallenge(req: Request, res: Response, next: NextFunction) {
    try {
        const { walletAddress } = req.body;
        res.json({ success: true, data: { nonce: "TODO", expiresAt: new Date() } });
    } catch (err) { next(err); }
}

/**
 * POST /api/wallets
 */
export async function linkWallet(req: Request, res: Response, next: NextFunction) {
    try {
        const { walletAddress, nonce, signature, label } = req.body;
        res.status(201).json({ success: true, data: null });
    } catch (err) { next(err); }
}

/**
 * DELETE /api/wallets/:id
 */
export async function unlinkWallet(req: Request, res: Response, next: NextFunction) {
    try {
        res.status(204).send();
    } catch (err) { next(err); }
}

/**
 * PUT /api/wallets/:id/primary
 */
export async function setPrimaryWallet(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

/**
 * PATCH /api/wallets/:id
 */
export async function updateWallet(req: Request, res: Response, next: NextFunction) {
    try {
        const { label } = req.body;
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

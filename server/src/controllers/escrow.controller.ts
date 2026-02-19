import { Request, Response, NextFunction } from "express";

// TODO: Inject EscrowService

/**
 * POST /api/escrows
 * Creates an escrow and returns unsigned Soroban XDR.
 */
export async function createEscrow(req: Request, res: Response, next: NextFunction) {
    try {
        const { buyerId, sellerId, amount, assetCode, expiresAt } = req.body;
        // const result = await escrowService.createEscrow(req.body);
        res.status(201).json({ success: true, data: { escrowId: "TODO", xdr: "BASE64_XDR" } });
    } catch (err) { next(err); }
}

/**
 * GET /api/escrows
 * List with optional filters: buyerId, sellerId, status.
 */
export async function listEscrows(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: [] });
    } catch (err) { next(err); }
}

/**
 * GET /api/escrows/:id
 */
export async function getEscrow(req: Request, res: Response, next: NextFunction) {
    try {
        const { id } = req.params;
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

/**
 * POST /api/escrows/webhook
 * Receives on-chain status updates. Validates X-Webhook-Secret header.
 */
export async function webhookEscrowUpdate(req: Request, res: Response, next: NextFunction) {
    try {
        const secret = req.headers["x-webhook-secret"];
        if (!process.env.WEBHOOK_SECRET || secret !== process.env.WEBHOOK_SECRET) {
            return res.status(process.env.WEBHOOK_SECRET ? 401 : 503).json({
                success: false,
                error: process.env.WEBHOOK_SECRET ? "Unauthorized" : "Webhook not configured",
            });
        }
        // await escrowService.processEscrowEvent(req.body);
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

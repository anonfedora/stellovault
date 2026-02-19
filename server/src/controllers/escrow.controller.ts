import { Request, Response, NextFunction } from "express";
import { env } from "../config/env";
import escrowService from "../services/escrow.service";

/**
 * POST /api/escrows
 * Creates an escrow and returns unsigned Soroban XDR.
 */
export async function createEscrow(req: Request, res: Response, next: NextFunction) {
    try {
        const result = await escrowService.createEscrow(req.body);
        res.status(201).json({ success: true, data: result });
    } catch (err) { next(err); }
}

/**
 * GET /api/escrows
 * List with optional filters: buyerId, sellerId, status.
 */
export async function listEscrows(req: Request, res: Response, next: NextFunction) {
    try {
        const { buyerId, sellerId, status, page, limit } = req.query;
        const result = await escrowService.listEscrows({
            buyerId: typeof buyerId === "string" ? buyerId : undefined,
            sellerId: typeof sellerId === "string" ? sellerId : undefined,
            status: typeof status === "string" ? status : undefined,
            page: typeof page === "string" ? page : undefined,
            limit: typeof limit === "string" ? limit : undefined,
        });
        res.json({ success: true, data: result });
    } catch (err) { next(err); }
}

/**
 * GET /api/escrows/:id
 */
export async function getEscrow(req: Request, res: Response, next: NextFunction) {
    try {
        const escrow = await escrowService.getEscrow(req.params.id);
        res.json({ success: true, data: escrow });
    } catch (err) { next(err); }
}

/**
 * POST /api/escrows/webhook
 * Receives on-chain status updates. Validates X-Webhook-Secret header.
 */
export async function webhookEscrowUpdate(req: Request, res: Response, next: NextFunction) {
    try {
        const secret = req.headers["x-webhook-secret"];
        const configuredSecret = env.webhookSecret;
        if (!configuredSecret) {
            return res.status(503).json({
                success: false,
                error: "Webhook not configured",
            });
        }
        if (secret !== configuredSecret) {
            return res.status(401).json({
                success: false,
                error: "Unauthorized",
            });
        }
        const updatedEscrow = await escrowService.processEscrowEvent(req.body);
        res.json({ success: true, data: updatedEscrow });
    } catch (err) { next(err); }
}

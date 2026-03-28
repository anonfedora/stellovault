import { Request, Response, NextFunction } from "express";
import { env } from "../config/env";
import escrowService from "../services/escrow.service";
import { verifyWebhookSignature } from "../services/webhook-signature.service";

/**
 * POST /api/escrows
 * Creates an escrow and returns unsigned Soroban XDR.
 */
export async function createEscrow(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const result = await escrowService.createEscrow(req.body);
    res.status(201).json({ success: true, data: result });
  } catch (err) {
    next(err);
  }
}

/**
 * GET /api/escrows
 * List with optional filters: buyerId, sellerId, status.
 */
export async function listEscrows(
  req: Request,
  res: Response,
  next: NextFunction,
) {
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
  } catch (err) {
    next(err);
  }
}

/**
 * GET /api/escrows/:id
 */
export async function getEscrow(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const escrow = await escrowService.getEscrow(req.params.id);
    res.json({ success: true, data: escrow });
  } catch (err) {
    next(err);
  }
}

export async function getEscrowStatus(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const status = await escrowService.getEscrowStatus(req.params.id);
    res.json({ success: true, data: status });
  } catch (err) {
    next(err);
  }
}

/**
 * POST /api/escrows/webhook
 * Receives on-chain status updates. Validates X-Webhook-Secret header.
 */
export async function webhookEscrowUpdate(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const configuredSecret = env.webhookSignatureSecret || env.webhookSecret;
    if (!configuredSecret) {
      return res.status(503).json({
        success: false,
        error: "Webhook not configured",
      });
    }

    const signatureHeader = req.headers["x-webhook-signature"];
    const signature = Array.isArray(signatureHeader)
      ? signatureHeader[0]
      : signatureHeader;
    const rawBody =
      (req as Request & { rawBody?: string }).rawBody ??
      JSON.stringify(req.body ?? {});
    if (!verifyWebhookSignature(rawBody, signature, configuredSecret)) {
      return res.status(401).json({
        success: false,
        error: "Invalid webhook signature",
      });
    }
    const updatedEscrow = await escrowService.processEscrowEvent(req.body);
    res.json({ success: true, data: updatedEscrow });
  } catch (err) {
    next(err);
  }
}

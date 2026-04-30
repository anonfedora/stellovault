import { NextFunction, Request, Response } from "express";
import { UnauthorizedError, ValidationError } from "../config/errors";
import webhookService from "../services/webhook.service";

function sanitizeWebhook(webhook: Record<string, unknown>) {
  const { secret: _secret, encryptionKey: _encryptionKey, ...safeWebhook } = webhook;
  return {
    ...safeWebhook,
    hasSecret: Boolean(_secret),
    hasEncryptionKey: Boolean(_encryptionKey),
  };
}

function requireUserId(req: Request): string {
  const userId = req.user?.userId;
  if (!userId) {
    throw new UnauthorizedError("Authentication required");
  }
  return userId;
}

export async function registerWebhook(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const userId = requireUserId(req);
    const { url, events, secret, authMethod, authConfig, encryptionEnabled, encryptionKey, rateLimitPerMinute, isActive } =
      req.body ?? {};

    if (typeof url !== "string" || !Array.isArray(events)) {
      throw new ValidationError("url (string) and events (string[]) are required");
    }

    const webhook = await webhookService.registerWebhook(userId, url, events, secret, {
      authMethod,
      authConfig,
      encryptionEnabled,
      encryptionKey,
      rateLimitPerMinute,
      isActive,
    });

    res.status(201).json({ success: true, data: sanitizeWebhook(webhook as unknown as Record<string, unknown>) });
  } catch (error) {
    next(error);
  }
}

export async function listWebhooks(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const userId = requireUserId(req);
    const webhooks = await webhookService.listWebhooks(userId);
    res.json({
      success: true,
      data: webhooks.map((webhook) =>
        sanitizeWebhook(webhook as unknown as Record<string, unknown>),
      ),
    });
  } catch (error) {
    next(error);
  }
}

export async function updateWebhook(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const userId = requireUserId(req);
    const webhook = await webhookService.updateWebhook(userId, req.params.id, req.body ?? {});
    res.json({
      success: true,
      data: sanitizeWebhook(webhook as unknown as Record<string, unknown>),
    });
  } catch (error) {
    next(error);
  }
}

export async function deleteWebhook(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const userId = requireUserId(req);
    await webhookService.deleteWebhook(userId, req.params.id);
    res.status(204).send();
  } catch (error) {
    next(error);
  }
}

export async function testWebhook(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const userId = requireUserId(req);
    const deliveryId = await webhookService.testWebhook(userId, req.params.id);
    res.status(202).json({
      success: true,
      data: {
        deliveryId,
        message: "Webhook test queued",
      },
    });
  } catch (error) {
    next(error);
  }
}

export async function getWebhookLogs(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const userId = requireUserId(req);
    const limitQuery = req.query.limit;
    const limit =
      typeof limitQuery === "string" && Number.isFinite(Number(limitQuery))
        ? Number(limitQuery)
        : 50;

    const logs = await webhookService.getWebhookLogs(userId, req.params.id, limit);
    res.json({ success: true, data: logs });
  } catch (error) {
    next(error);
  }
}

export async function getWebhookMetrics(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  try {
    const userId = requireUserId(req);
    const metrics = await webhookService.getWebhookMetrics(req.params.id, userId);
    res.json({ success: true, data: metrics });
  } catch (error) {
    next(error);
  }
}

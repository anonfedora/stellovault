import { randomUUID } from "crypto";
import { Request, Response, NextFunction } from "express";
import logger from "../config/logger";

declare global {
  namespace Express {
    interface Request {
      requestId?: string;
      correlationId?: string;
    }
  }
}

export function requestTraceMiddleware(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  const start = Date.now();
  const { method, url } = req;
  const inboundRequestId = req.header("x-request-id");
  const inboundCorrelationId = req.header("x-correlation-id");

  req.requestId = inboundRequestId || randomUUID();
  req.correlationId = inboundCorrelationId || req.requestId;
  res.setHeader("x-request-id", req.requestId);
  res.setHeader("x-correlation-id", req.correlationId);

  res.on("finish", () => {
    logger.info("http_request", {
      requestId: req.requestId,
      correlationId: req.correlationId,
      method,
      path: url,
      statusCode: res.statusCode,
      durationMs: Date.now() - start,
      userAgent: req.header("user-agent"),
      ip: req.ip,
    });
  });

  next();
}

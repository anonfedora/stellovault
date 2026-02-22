import { Request, Response, NextFunction } from "express";

/**
 * Logs request method, path, status code, and duration.
 */
export function requestTraceMiddleware(
  req: Request,
  res: Response,
  next: NextFunction,
) {
  const start = Date.now();
  const { method, url } = req;

  res.on("finish", () => {
    const duration = Date.now() - start;
    const { statusCode } = res;
    console.log(`[TRACE] ${method} ${url} → ${statusCode} (${duration}ms)`);
  });

  next();
}

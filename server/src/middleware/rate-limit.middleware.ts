import { Request, Response, NextFunction } from "express";

const WINDOW_MS = 60_000; // 1 minute
const MAX_REQUESTS = 100;

const requestCounts = new Map<string, { count: number; resetAt: number }>();

/**
 * Simple in-process rate limiter: 100 requests/min per IP.
 * Replace with `express-rate-limit` + Redis in production.
 */
export function rateLimitMiddleware(req: Request, res: Response, next: NextFunction) {
    const ip = req.ip || "unknown";
    const now = Date.now();
    const entry = requestCounts.get(ip);

    if (!entry || now > entry.resetAt) {
        requestCounts.set(ip, { count: 1, resetAt: now + WINDOW_MS });
        return next();
    }

    entry.count++;
    if (entry.count > MAX_REQUESTS) {
        res.setHeader("Retry-After", String(Math.ceil((entry.resetAt - now) / 1000)));
        return res.status(429).json({ success: false, error: "Too many requests" });
    }

    next();
}

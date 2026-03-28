import { Request, Response, NextFunction } from "express";
import rateLimit from "express-rate-limit";
import jwt from "jsonwebtoken";
import { env } from "../config/env";
import { JwtPayload } from "./auth.middleware";

// ── GeoIP (optional dependency — activates when geoip-lite is installed) ──────
type GeoLookup = (ip: string) => { country: string } | null;
let geoipLookup: GeoLookup | null = null;
try {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const geoip = require("geoip-lite") as { lookup: GeoLookup };
  geoipLookup = (ip: string) => geoip.lookup(ip);
  console.log("[GEO BLOCK] geoip-lite loaded — jurisdiction blocking active");
} catch {
  console.warn(
    "[GEO BLOCK] geoip-lite not installed. Jurisdiction blocking is disabled. " +
      "Run `npm install` after freeing disk space to enable."
  );
}

/**
 * OFAC-sanctioned jurisdictions. Requests originating from these countries
 * are blocked at the network layer before any business logic is reached.
 */
const SANCTIONED_COUNTRIES = new Set([
  "CU", // Cuba
  "IR", // Iran
  "KP", // North Korea
  "SY", // Syria
  "RU", // Russia
  "BY", // Belarus
  "SD", // Sudan
  "MM", // Myanmar / Burma
  "SS", // South Sudan
  "CF", // Central African Republic
  "YE", // Yemen
  "ZW", // Zimbabwe
  "VE", // Venezuela
]);

// ── Suspicious IP tracking ────────────────────────────────────────────────────
interface SuspiciousEntry {
  failures: number;
  firstFailureAt: number;
  throttledUntil?: number;
}

const suspiciousIPs = new Map<string, SuspiciousEntry>();

const SUSPICIOUS_THRESHOLD = 5; // failures before throttling
const SUSPICIOUS_WINDOW_MS = 10 * 60 * 1000; // 10-minute observation window
const THROTTLE_DURATION_MS = 15 * 60 * 1000; // 15-minute throttle period

// Periodically evict expired entries to prevent unbounded memory growth
setInterval(
  () => {
    const now = Date.now();
    for (const [ip, entry] of suspiciousIPs) {
      const windowExpired = now - entry.firstFailureAt > SUSPICIOUS_WINDOW_MS;
      const throttleExpired = !entry.throttledUntil || entry.throttledUntil < now;
      if (windowExpired && throttleExpired) {
        suspiciousIPs.delete(ip);
      }
    }
  },
  30 * 60 * 1000 // run every 30 minutes
).unref(); // do not keep the process alive for this timer

/**
 * Records a failed authentication challenge attempt for an IP address.
 * Once the failure count within the observation window reaches
 * SUSPICIOUS_THRESHOLD the IP is temporarily throttled and a warning is emitted.
 */
export function recordFailedAuthChallenge(ip: string): void {
  if (!ip) return;

  const now = Date.now();
  const entry: SuspiciousEntry = suspiciousIPs.get(ip) ?? {
    failures: 0,
    firstFailureAt: now,
  };

  // Reset the window counter if the observation period has expired
  if (now - entry.firstFailureAt > SUSPICIOUS_WINDOW_MS) {
    entry.failures = 0;
    entry.firstFailureAt = now;
  }

  entry.failures += 1;

  if (entry.failures >= SUSPICIOUS_THRESHOLD) {
    entry.throttledUntil = now + THROTTLE_DURATION_MS;
    console.warn(
      `[SUSPICIOUS ACTIVITY] IP ${ip} flagged — ${entry.failures} failed auth challenges ` +
        `within the observation window. Throttled until ${new Date(entry.throttledUntil).toISOString()}`
    );
  }

  suspiciousIPs.set(ip, entry);
}

function isThrottled(ip: string): boolean {
  const entry = suspiciousIPs.get(ip);
  if (!entry?.throttledUntil) return false;
  return entry.throttledUntil > Date.now();
}

// ── Rate limiters ─────────────────────────────────────────────────────────────

/** Strict limiter applied to suspicious / throttled IPs: 10 req/min. */
const suspiciousLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 10,
  standardHeaders: true,
  legacyHeaders: false,
  handler: (_req, res) => {
    res.status(429).json({
      success: false,
      error: "Too many requests — suspicious activity detected",
    });
  },
});

/** Public (unauthenticated) limiter: 100 req/min per IP. */
const publicRateLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 100,
  standardHeaders: true,
  legacyHeaders: false,
  handler: (_req, res) => {
    res.status(429).json({ success: false, error: "Too many requests" });
  },
});

/** Authenticated limiter: 500 req/min keyed by userId. */
const authenticatedRateLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 500,
  standardHeaders: true,
  legacyHeaders: false,
  keyGenerator: (req) =>
    (req as Request & { _rlUserId?: string })._rlUserId ?? req.ip ?? "unknown",
  handler: (_req, res) => {
    res.status(429).json({ success: false, error: "Too many requests" });
  },
});

// ── Exported middleware ───────────────────────────────────────────────────────

/**
 * Blocks requests originating from OFAC-sanctioned jurisdictions.
 * Loopback / private addresses are always allowed through.
 * Requires the optional `geoip-lite` package to be installed.
 */
export function geoIpBlockMiddleware(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  if (!geoipLookup) {
    return next(); // GeoIP disabled — skip
  }

  const ip = req.ip ?? "";

  // Always allow loopback and unresolvable addresses
  if (!ip || ip === "::1" || ip === "127.0.0.1" || ip.startsWith("::ffff:127.")) {
    return next();
  }

  const geo = geoipLookup(ip);
  if (geo && SANCTIONED_COUNTRIES.has(geo.country)) {
    console.warn(
      `[GEO BLOCK] Blocked request from sanctioned country ${geo.country} (IP: ${ip})`
    );
    res.status(403).json({
      success: false,
      error: "Access from your region is not permitted",
    });
    return;
  }

  next();
}

/**
 * Tiered rate limiter:
 *  - Throttled / suspicious IPs → 10 req/min
 *  - Authenticated (valid JWT)  → 500 req/min  (keyed by userId)
 *  - Public (no / invalid JWT)  → 100 req/min  (keyed by IP)
 *
 * Auth status is determined by peeking at the JWT signature only —
 * no database session check is performed at this layer.
 */
export function tieredRateLimitMiddleware(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const ip = req.ip ?? "unknown";

  // Throttle IPs that have been flagged for suspicious auth behaviour
  if (isThrottled(ip)) {
    return suspiciousLimiter(req, res, next);
  }

  // Peek at the Authorization header to determine the tier
  const authHeader = req.headers.authorization;
  if (authHeader?.startsWith("Bearer ")) {
    try {
      const payload = jwt.verify(
        authHeader.slice(7),
        env.jwt.accessSecret
      ) as JwtPayload;
      // Stash the userId so the keyGenerator can use it without re-parsing
      (req as Request & { _rlUserId?: string })._rlUserId = payload.userId;
      return authenticatedRateLimiter(req, res, next);
    } catch {
      // Invalid or expired token — fall through to the public limiter
    }
  }

  publicRateLimiter(req, res, next);
}

// Keep the old export name as an alias so callers that import rateLimitMiddleware
// still compile without changes (app.ts will be updated to use the new exports).
export const rateLimitMiddleware = tieredRateLimitMiddleware;

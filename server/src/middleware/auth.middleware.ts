import { Request, Response, NextFunction } from "express";
import jwt from "jsonwebtoken";
import { prisma } from "../config/prisma";
import { env } from "../config/env";
import { UnauthorizedError } from "../config/errors";

export interface JwtPayload {
    userId: string;
    jti: string;
    walletAddress: string;
}

// Extend Express Request to carry user info
declare global {
    namespace Express {
        interface Request {
            user?: JwtPayload;
        }
    }
}



/**
 * Extracts and verifies a Bearer JWT from the Authorization header.
 * Also checks that the session hasn't been revoked.
 * Attaches the decoded payload to `req.user`.
 */
export async function authMiddleware(req: Request, _res: Response, next: NextFunction) {
    const header = req.headers.authorization;
    if (!header?.startsWith("Bearer ")) {
        return next(new UnauthorizedError("Missing or malformed Authorization header"));
    }

    const token = header.slice(7);
    try {
        const payload = jwt.verify(token, env.jwt.accessSecret) as JwtPayload;

        // Check if session is revoked in the database
        const session = await prisma.session.findUnique({
            where: { jti: payload.jti },
        });

        if (!session || session.revoked) {
            return next(new UnauthorizedError("Session revoked or invalid"));
        }

        req.user = payload;
        next();
    } catch (err) {
        if (err instanceof UnauthorizedError) {
            return next(err);
        }
        next(new UnauthorizedError("Invalid or expired access token"));
    }
}


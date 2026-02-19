import { Request, Response, NextFunction } from "express";
import {
    NotFoundError,
    UnauthorizedError,
    ForbiddenError,
    ConflictError,
    ValidationError,
    TooManyRequestsError,
} from "../config/errors";

/**
 * Central error handler. Maps custom error classes to HTTP status codes.
 * Must be registered LAST in Express middleware chain.
 */
export function errorMiddleware(
    err: Error,
    _req: Request,
    res: Response,
    _next: NextFunction
) {
    let status = 500;

    if (err instanceof ValidationError) status = 400;
    else if (err instanceof UnauthorizedError) status = 401;
    else if (err instanceof ForbiddenError) status = 403;
    else if (err instanceof NotFoundError) status = 404;
    else if (err instanceof ConflictError) status = 409;
    else if (err instanceof TooManyRequestsError) status = 429;

    if (status === 500) {
        console.error("[ERROR]", err);
    }

    res.status(status).json({
        success: false,
        error: err.message || "Internal server error",
    });
}

/**
 * Catch-all for unmatched routes.
 */
export function notFoundMiddleware(_req: Request, _res: Response, next: NextFunction) {
    next(new NotFoundError("Route not found"));
}

/**
 * Custom application error classes.
 * Used by the central error middleware to map to HTTP status codes.
 */

export class NotFoundError extends Error {
    constructor(message = "Not found") {
        super(message);
        this.name = "NotFoundError";
    }
}

export class UnauthorizedError extends Error {
    constructor(message = "Unauthorized") {
        super(message);
        this.name = "UnauthorizedError";
    }
}

export class ForbiddenError extends Error {
    constructor(message = "Forbidden") {
        super(message);
        this.name = "ForbiddenError";
    }
}

export class ConflictError extends Error {
    constructor(message = "Conflict") {
        super(message);
        this.name = "ConflictError";
    }
}

export class ValidationError extends Error {
    constructor(message = "Validation error") {
        super(message);
        this.name = "ValidationError";
    }
}

export class TooManyRequestsError extends Error {
    constructor(message = "Too many requests") {
        super(message);
        this.name = "TooManyRequestsError";
    }
}

import { Request, Response, NextFunction } from "express";
import { prisma } from "../config/prisma";

/**
 * Middleware to log sensitive operations to the AuditLog table.
 * 
 * @param action The name of the action being performed (e.g., 'LOAN_APPROVAL')
 * @param getResourceId An optional function to extract the resource ID from the request
 */
export function auditLog(action: string, getResourceId?: (req: Request) => string) {
    return async (req: Request, _res: Response, next: NextFunction) => {
        const userId = req.user?.userId;
        const resourceId = getResourceId ? getResourceId(req) : req.params.id || req.body.id;
        const ipAddress = req.ip || req.socket.remoteAddress || null;
        
        // Sanitize payload: remove sensitive fields like passwords, private keys if they exist
        const payload = { ...req.body };
        const sensitiveFields = ["password", "privateKey", "secret", "token"];
        sensitiveFields.forEach(field => delete payload[field]);

        try {
            await prisma.auditLog.create({
                data: {
                    userId,
                    action,
                    resourceId: resourceId?.toString(),
                    ipAddress,
                    payload,
                },
            });
        } catch (error) {
            console.error(`Failed to create audit log for action ${action}:`, error);
            // We don't call next(error) here because we don't want to block the actual operation 
            // if logging fails, unless specified otherwise.
        }

        next();
    };
}

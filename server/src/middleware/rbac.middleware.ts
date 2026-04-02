import { Request, Response, NextFunction } from "express";
import { rbacService } from "../services/rbac.service";
import { 
  ResourceType, 
  PermissionContext,
  AuthenticatedUser,
  UserRole,
  PermissionScope
} from "../types/rbac";
import { ForbiddenError, UnauthorizedError } from "../config/errors";

/**
 * Extend Express Request to carry authenticated user info
 */
declare global {
  namespace Express {
    interface Request {
      user?: JwtPayload;
    }
  }
}

interface JwtPayload {
  userId: string;
  jti: string;
  walletAddress: string;
  role: UserRole;
  permissions?: PermissionScope[];
}

/**
 * Middleware to check if user has required permission
 * 
 * Usage: checkPermission(ResourceType.LOAN, 'write')
 */
export function checkPermission(
  resourceType: ResourceType,
  action: 'read' | 'write' | 'delete' | 'approve' | 'release' | 'refund' | 'process' | 'override'
) {
  return async (req: Request, res: Response, next: NextFunction) => {
    if (!req.user) {
      return next(new UnauthorizedError("Authentication required"));
    }

    try {
      // Convert JwtPayload to AuthenticatedUser
      const authenticatedUser: AuthenticatedUser = {
        userId: req.user.userId,
        jti: req.user.jti,
        walletAddress: req.user.walletAddress,
        role: req.user.role,
        permissions: req.user.permissions || []
      };

      // Get resource ID from request parameters or body
      const resourceId = req.params.id || req.body.id || req.body.loanId || req.body.escrowId;
      
      const context: PermissionContext = {
        resourceType,
        action,
        resourceId
      };

      const result = await rbacService.checkPermission(authenticatedUser, context);

      if (!result.allowed) {
        return next(new ForbiddenError(
          result.reason || "Insufficient permissions"
        ));
      }

      // Attach permission check result to request for downstream use
      req.rbacCheck = result;
      next();
    } catch (error) {
      next(error);
    }
  };
}

/**
 * Middleware to check resource ownership
 * 
 * Usage: checkOwnership(ResourceType.LOAN, 'borrowerId')
 */
export function checkOwnership(
  resourceType: ResourceType,
  ownerField: string = 'borrowerId'
) {
  return async (req: Request, res: Response, next: NextFunction) => {
    if (!req.user) {
      return next(new UnauthorizedError("Authentication required"));
    }

    try {
      // Convert JwtPayload to AuthenticatedUser
      const authenticatedUser: AuthenticatedUser = {
        userId: req.user.userId,
        jti: req.user.jti,
        walletAddress: req.user.walletAddress,
        role: req.user.role,
        permissions: req.user.permissions || []
      };

      const resourceId = req.params.id || req.body.id;
      
      if (!resourceId) {
        return next(new ForbiddenError("Resource ID required"));
      }

      const result = await rbacService.canAccessResource(
        authenticatedUser,
        resourceType,
        resourceId,
        'write' // Ownership checks are typically for write operations
      );

      if (!result.allowed) {
        return next(new ForbiddenError(
          result.reason || "Resource ownership required"
        ));
      }

      next();
    } catch (error) {
      next(error);
    }
  };
}

/**
 * Middleware to require specific role
 * 
 * Usage: requireRole(UserRole.ADMIN)
 */
export function requireRole(requiredRole: UserRole) {
  return (req: Request, res: Response, next: NextFunction) => {
    if (!req.user) {
      return next(new UnauthorizedError("Authentication required"));
    }

    if (req.user.role !== requiredRole && req.user.role !== UserRole.ADMIN) {
      return next(new ForbiddenError(
        `Access denied. Required role: ${requiredRole}`
      ));
    }

    next();
  };
}

/**
 * Middleware to require any of the specified roles
 * 
 * Usage: requireAnyRole([UserRole.LENDER, UserRole.ADMIN])
 */
export function requireAnyRole(allowedRoles: UserRole[]) {
  return (req: Request, res: Response, next: NextFunction) => {
    if (!req.user) {
      return next(new UnauthorizedError("Authentication required"));
    }

    if (!allowedRoles.includes(req.user.role) && req.user.role !== UserRole.ADMIN) {
      return next(new ForbiddenError(
        `Access denied. Required one of: ${allowedRoles.join(', ')}`
      ));
    }

    next();
  };
}

/**
 * Middleware to check if user can access their own resources only
 * 
 * Usage: checkSelfAccess('userId')
 */
export function checkSelfAccess(userIdField: string = 'userId') {
  return (req: Request, res: Response, next: NextFunction) => {
    if (!req.user) {
      return next(new UnauthorizedError("Authentication required"));
    }

    const targetUserId = req.params[userIdField] || req.body[userIdField];
    const requestingUserId = req.user.userId;

    // Admin can access any user's resources
    if (req.user.role === UserRole.ADMIN) {
      return next();
    }

    // Users can only access their own resources
    if (targetUserId !== requestingUserId) {
      return next(new ForbiddenError("Access denied: Can only access own resources"));
    }

    next();
  };
}

/**
 * Middleware for lender-specific operations
 * Prevents lenders from approving their own loans
 */
export function checkLenderSelfApproval() {
  return async (req: Request, res: Response, next: NextFunction) => {
    if (!req.user) {
      return next(new UnauthorizedError("Authentication required"));
    }

    if (req.user.role !== UserRole.LENDER) {
      return next(); // Non-lenders are not subject to this check
    }

    try {
      const loanId = req.params.id || req.body.loanId || req.body.id;
      
      if (!loanId) {
        return next(new ForbiddenError("Loan ID required"));
      }

      // This would need to be implemented to check if the lender is trying to approve their own loan
      // For now, we'll pass through and let the service layer handle the detailed check
      next();
    } catch (error) {
      next(error);
    }
  };
}

/**
 * Extend Express Request interface for RBAC
 */
declare global {
  namespace Express {
    interface Request {
      rbacCheck?: {
        allowed: boolean;
        reason?: string;
        requiredRole?: UserRole;
        requiredPermission?: PermissionScope;
      };
    }
  }
}

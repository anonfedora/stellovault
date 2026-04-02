import { prisma } from "../config/prisma";
import { 
  UserRole, 
  PermissionScope, 
  ResourceType, 
  PermissionContext,
  PermissionCheckResult,
  AuthenticatedUser,
  hasPermission,
  isResourceOwner
} from "../types/rbac";
import { ForbiddenError, NotFoundError } from "../config/errors";

/**
 * Role-Based Access Control (RBAC) Service
 * 
 * Handles permission checking, role management, and resource ownership validation
 */
export class RbacService {
  /**
   * Check if a user has permission to perform an action on a resource
   */
  async checkPermission(
    user: AuthenticatedUser,
    context: PermissionContext
  ): Promise<PermissionCheckResult> {
    const { resourceType, resourceId, action, resourceOwnerId } = context;

    // Admin bypass - has all permissions
    if (user.role === UserRole.ADMIN) {
      return { allowed: true };
    }

    // Build permission scope from context
    const requiredScope = this.buildPermissionScope(resourceType, action);

    // Check role-based permissions
    const hasRolePermission = hasPermission(user.role, requiredScope);
    
    // Check user-specific permissions (if any)
    const hasUserPermission = await this.checkUserPermission(user.userId, requiredScope);

    // Check resource ownership for write operations
    const isOwner = resourceOwnerId ? 
      isResourceOwner(user.walletAddress, resourceOwnerId) : false;

    // Determine if access is allowed
    let allowed = false;
    let reason: string | undefined;

    if (hasRolePermission || hasUserPermission) {
      // For read operations, any permission is sufficient
      if (action === 'read') {
        allowed = true;
      } else {
        // For write operations, must be owner or have admin-level permissions
        if (isOwner || this.isAdminLevelPermission(requiredScope)) {
          allowed = true;
        } else {
          reason = "Resource ownership required for write operations";
        }
      }
    } else {
      reason = "Insufficient permissions";
    }

    return {
      allowed,
      reason: allowed ? undefined : reason,
      requiredRole: !allowed ? this.getRequiredRole(resourceType, action) : undefined,
      requiredPermission: !allowed ? requiredScope : undefined
    };
  }

  /**
   * Check if a user can access a specific resource (with ownership validation)
   */
  async canAccessResource(
    user: AuthenticatedUser,
    resourceType: ResourceType,
    resourceId: string,
    action: 'read' | 'write' | 'delete'
  ): Promise<PermissionCheckResult> {
    // Get resource and owner information
    const resourceInfo = await this.getResourceInfo(resourceType, resourceId);
    
    if (!resourceInfo) {
      return { allowed: false, reason: "Resource not found" };
    }

    const context: PermissionContext = {
      resourceType,
      resourceId,
      action,
      resourceOwnerId: resourceInfo.ownerId
    };

    return this.checkPermission(user, context);
  }

  /**
   * Get user's effective permissions (role + user-specific)
   */
  async getUserPermissions(userId: string): Promise<PermissionScope[]> {
    const user = await prisma.user.findUnique({
      where: { id: userId },
      include: {
        assignedRole: {
          include: {
            rolePermissions: {
              include: { permission: true }
            }
          }
        },
        userPermissions: {
          include: { permission: true }
        }
      }
    });

    if (!user) {
      throw new NotFoundError("User not found");
    }

    // Get role permissions
    const rolePermissions = user.assignedRole?.rolePermissions.map(
      (rp: any) => rp.permission.scope as PermissionScope
    ) || [];

    // Get user-specific permissions
    const userPermissions = user.userPermissions.map(
      (up: any) => up.permission.scope as PermissionScope
    );

    // Combine and deduplicate
    const allPermissions = [...new Set([...rolePermissions, ...userPermissions])];

    return allPermissions;
  }

  /**
   * Grant a specific permission to a user
   */
  async grantUserPermission(
    userId: string,
    permissionScope: PermissionScope,
    grantedBy: string
  ): Promise<void> {
    // For now, this is a placeholder until the RBAC models are created
    // TODO: Implement after migration
    console.log(`Granting permission ${permissionScope} to user ${userId} by ${grantedBy}`);
  }

  /**
   * Revoke a specific permission from a user
   */
  async revokeUserPermission(userId: string, permissionScope: PermissionScope): Promise<void> {
    // For now, this is a placeholder until the RBAC models are created
    // TODO: Implement after migration
    console.log(`Revoking permission ${permissionScope} from user ${userId}`);
  }

  /**
   * Assign a role to a user
   */
  async assignRole(userId: string, roleName: UserRole): Promise<void> {
    // For now, this is a placeholder until the RBAC models are created
    // TODO: Implement after migration
    console.log(`Assigning role ${roleName} to user ${userId}`);
  }

  // --- PRIVATE HELPERS ---

  /**
   * Build permission scope from resource type and action
   */
  private buildPermissionScope(resourceType: ResourceType, action: string): PermissionScope {
    const scopeMap: Record<ResourceType, Record<string, PermissionScope>> = {
      [ResourceType.LOAN]: {
        'read': PermissionScope.LOAN_READ,
        'write': PermissionScope.LOAN_WRITE,
        'delete': PermissionScope.LOAN_DELETE,
        'approve': PermissionScope.LOAN_APPROVE
      },
      [ResourceType.ESCROW]: {
        'read': PermissionScope.ESCROW_READ,
        'write': PermissionScope.ESCROW_WRITE,
        'release': PermissionScope.ESCROW_RELEASE,
        'refund': PermissionScope.ESCROW_REFUND
      },
      [ResourceType.PAYMENT]: {
        'read': PermissionScope.PAYMENT_READ,
        'write': PermissionScope.PAYMENT_WRITE,
        'process': PermissionScope.PAYMENT_PROCESS
      },
      [ResourceType.USER]: {
        'read': PermissionScope.USER_READ,
        'write': PermissionScope.USER_WRITE,
        'delete': PermissionScope.USER_DELETE
      },
      [ResourceType.AUDIT]: {
        'read': PermissionScope.AUDIT_READ
      }
    };

    return scopeMap[resourceType]?.[action] as PermissionScope;
  }

  /**
   * Check if permission is admin-level
   */
  private isAdminLevelPermission(permission: PermissionScope): boolean {
    const adminPermissions = [
      PermissionScope.LOAN_DELETE,
      PermissionScope.LOAN_APPROVE,
      PermissionScope.ESCROW_RELEASE,
      PermissionScope.ESCROW_REFUND,
      PermissionScope.PAYMENT_PROCESS,
      PermissionScope.USER_DELETE,
      PermissionScope.RISK_OVERRIDE,
      PermissionScope.ADMIN_ALL
    ];

    return adminPermissions.includes(permission);
  }

  /**
   * Get required role for a resource/action combination
   */
  private getRequiredRole(resourceType: ResourceType, action: string): UserRole {
    const requirements: Record<ResourceType, Record<string, UserRole>> = {
      [ResourceType.LOAN]: {
        'write': UserRole.LENDER,
        'approve': UserRole.LENDER,
        'delete': UserRole.ADMIN
      },
      [ResourceType.ESCROW]: {
        'write': UserRole.BUYER,
        'release': UserRole.SELLER,
        'refund': UserRole.ADMIN
      },
      [ResourceType.PAYMENT]: {
        'write': UserRole.BUYER,
        'process': UserRole.LENDER
      },
      [ResourceType.USER]: {
        'write': UserRole.ADMIN,
        'delete': UserRole.ADMIN
      },
      [ResourceType.AUDIT]: {
        'read': UserRole.AUDITOR
      }
    };

    return requirements[resourceType]?.[action] || UserRole.ADMIN;
  }

  /**
   * Check user-specific permission
   */
  private async checkUserPermission(userId: string, permissionScope: PermissionScope): Promise<boolean> {
    const userPermission = await prisma.userPermission.findFirst({
      where: {
        userId,
        permission: {
          scope: permissionScope
        }
      }
    });

    return !!userPermission;
  }

  /**
   * Get resource information including owner
   */
  private async getResourceInfo(resourceType: ResourceType, resourceId: string): Promise<{ ownerId: string } | null> {
    switch (resourceType) {
      case ResourceType.LOAN:
        const loan = await prisma.loan.findUnique({
          where: { id: resourceId },
          select: { borrowerId: true, lenderId: true }
        });
        return loan ? { ownerId: loan.borrowerId } : null;

      case ResourceType.ESCROW:
        const escrow = await prisma.escrow.findUnique({
          where: { id: resourceId },
          select: { buyerId: true, sellerId: true }
        });
        return escrow ? { ownerId: escrow.buyerId } : null;

      case ResourceType.PAYMENT:
        const payment = await prisma.payment.findUnique({
          where: { id: resourceId },
          select: { loan: { select: { borrowerId: true } } }
        });
        return payment ? { ownerId: payment.loan.borrowerId } : null;

      case ResourceType.USER:
        const user = await prisma.user.findUnique({
          where: { id: resourceId },
          select: { stellarAddress: true }
        });
        return user ? { ownerId: user.stellarAddress } : null;

      default:
        return null;
    }
  }
}

export const rbacService = new RbacService();

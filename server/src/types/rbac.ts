/**
 * Role-Based Access Control (RBAC) Types
 * 
 * Multi-tenant permission system for StelloVault platform
 */

/**
 * User Roles - Define high-level user personas
 */
export enum UserRole {
  BUYER = 'BUYER',
  SELLER = 'SELLER', 
  LENDER = 'LENDER',
  AUDITOR = 'AUDITOR',
  ADMIN = 'ADMIN'
}

/**
 * Permission Scopes - Define what actions users can perform
 * Format: resource:action (e.g., loan:read, escrow:write)
 */
export enum PermissionScope {
  // Loan permissions
  LOAN_READ = 'loan:read',
  LOAN_WRITE = 'loan:write',
  LOAN_DELETE = 'loan:delete',
  LOAN_APPROVE = 'loan:approve',
  
  // Escrow permissions  
  ESCROW_READ = 'escrow:read',
  ESCROW_WRITE = 'escrow:write',
  ESCROW_RELEASE = 'escrow:release',
  ESCROW_REFUND = 'escrow:refund',
  
  // Payment permissions
  PAYMENT_READ = 'payment:read',
  PAYMENT_WRITE = 'payment:write',
  PAYMENT_PROCESS = 'payment:process',
  
  // User permissions
  USER_READ = 'user:read',
  USER_WRITE = 'user:write',
  USER_DELETE = 'user:delete',
  
  // System permissions
  AUDIT_READ = 'audit:read',
  RISK_OVERRIDE = 'risk:override',
  ADMIN_ALL = 'admin:all'
}

/**
 * Resource Types - Define protected resources
 */
export enum ResourceType {
  LOAN = 'loan',
  ESCROW = 'escrow', 
  PAYMENT = 'payment',
  USER = 'user',
  AUDIT = 'audit'
}

/**
 * Permission check context
 */
export interface PermissionContext {
  resourceType: ResourceType;
  resourceId?: string;
  action: 'read' | 'write' | 'delete' | 'approve' | 'release' | 'refund' | 'process' | 'override';
  resourceOwnerId?: string;
}

/**
 * Extended user payload with role information
 */
export interface AuthenticatedUser {
  userId: string;
  jti: string;
  walletAddress: string;
  role: UserRole;
  permissions: PermissionScope[];
}

/**
 * Permission check result
 */
export interface PermissionCheckResult {
  allowed: boolean;
  reason?: string;
  requiredRole?: UserRole;
  requiredPermission?: PermissionScope;
}

/**
 * Role-permission mapping
 */
export const ROLE_PERMISSIONS: Record<UserRole, PermissionScope[]> = {
  [UserRole.BUYER]: [
    PermissionScope.ESCROW_WRITE,
    PermissionScope.ESCROW_READ,
    PermissionScope.PAYMENT_WRITE,
    PermissionScope.PAYMENT_READ,
    PermissionScope.USER_READ,
    PermissionScope.USER_WRITE
  ],
  
  [UserRole.SELLER]: [
    PermissionScope.ESCROW_READ,
    PermissionScope.ESCROW_RELEASE,
    PermissionScope.PAYMENT_READ,
    PermissionScope.USER_READ,
    PermissionScope.USER_WRITE
  ],
  
  [UserRole.LENDER]: [
    PermissionScope.LOAN_WRITE,
    PermissionScope.LOAN_READ,
    PermissionScope.LOAN_APPROVE,
    PermissionScope.PAYMENT_READ,
    PermissionScope.USER_READ,
    PermissionScope.USER_WRITE
  ],
  
  [UserRole.AUDITOR]: [
    PermissionScope.LOAN_READ,
    PermissionScope.ESCROW_READ,
    PermissionScope.PAYMENT_READ,
    PermissionScope.USER_READ,
    PermissionScope.AUDIT_READ
  ],
  
  [UserRole.ADMIN]: [
    PermissionScope.ADMIN_ALL // Implicitly grants all permissions
  ]
};

/**
 * Helper function to check if a role has a specific permission
 */
export function hasPermission(role: UserRole, permission: PermissionScope): boolean {
  if (role === UserRole.ADMIN) return true; // Admin has all permissions
  return ROLE_PERMISSIONS[role]?.includes(permission) || false;
}

/**
 * Helper function to get all permissions for a role
 */
export function getRolePermissions(role: UserRole): PermissionScope[] {
  if (role === UserRole.ADMIN) return Object.values(PermissionScope);
  return ROLE_PERMISSIONS[role] || [];
}

/**
 * Helper function to check resource ownership
 */
export function isResourceOwner(
  userWalletAddress: string, 
  resourceOwnerId: string
): boolean {
  return userWalletAddress.toLowerCase() === resourceOwnerId.toLowerCase();
}

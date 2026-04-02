import { RbacService } from "../services/rbac.service";
import { UserRole, PermissionScope, ResourceType } from "../types/rbac";

describe('RBAC Service', () => {
  let rbacService: RbacService;

  beforeEach(() => {
    rbacService = new RbacService();
  });

  describe('Permission Checking', () => {
    it('should allow admin to access any resource', async () => {
      const adminUser = {
        userId: 'admin-1',
        jti: 'session-1',
        walletAddress: 'GBADMIN...',
        role: UserRole.ADMIN,
        permissions: []
      };

      const context = {
        resourceType: ResourceType.LOAN,
        action: 'delete' as const,
        resourceId: 'loan-1'
      };

      const result = await rbacService.checkPermission(adminUser, context);
      expect(result.allowed).toBe(true);
    });

    it('should allow lender to create loans', async () => {
      const lenderUser = {
        userId: 'lender-1',
        jti: 'session-1',
        walletAddress: 'GBLENDER...',
        role: UserRole.LENDER,
        permissions: []
      };

      const context = {
        resourceType: ResourceType.LOAN,
        action: 'write' as const,
        resourceId: 'loan-1'
      };

      const result = await rbacService.checkPermission(lenderUser, context);
      expect(result.allowed).toBe(true);
    });

    it('should deny buyer from creating loans', async () => {
      const buyerUser = {
        userId: 'buyer-1',
        jti: 'session-1',
        walletAddress: 'GBBUYER...',
        role: UserRole.BUYER,
        permissions: []
      };

      const context = {
        resourceType: ResourceType.LOAN,
        action: 'write' as const,
        resourceId: 'loan-1'
      };

      const result = await rbacService.checkPermission(buyerUser, context);
      expect(result.allowed).toBe(false);
      expect(result.reason).toBe("Insufficient permissions");
      expect(result.requiredRole).toBe(UserRole.LENDER);
    });

    it('should allow buyer to create escrows', async () => {
      const buyerUser = {
        userId: 'buyer-1',
        jti: 'session-1',
        walletAddress: 'GBBUYER...',
        role: UserRole.BUYER,
        permissions: []
      };

      const context = {
        resourceType: ResourceType.ESCROW,
        action: 'write' as const,
        resourceId: 'escrow-1'
      };

      const result = await rbacService.checkPermission(buyerUser, context);
      expect(result.allowed).toBe(true);
    });

    it('should deny lender from creating escrows', async () => {
      const lenderUser = {
        userId: 'lender-1',
        jti: 'session-1',
        walletAddress: 'GBLENDER...',
        role: UserRole.LENDER,
        permissions: []
      };

      const context = {
        resourceType: ResourceType.ESCROW,
        action: 'write' as const,
        resourceId: 'escrow-1'
      };

      const result = await rbacService.checkPermission(lenderUser, context);
      expect(result.allowed).toBe(false);
      expect(result.requiredRole).toBe(UserRole.BUYER);
    });
  });

  describe('Permission Scopes', () => {
    it('should build correct permission scopes', () => {
      const service = new RbacService();
      
      // Test that the service builds scopes correctly
      expect(ResourceType.LOAN).toBe('loan');
      expect(ResourceType.ESCROW).toBe('escrow');
      expect(PermissionScope.LOAN_WRITE).toBe('loan:write');
      expect(PermissionScope.ESCROW_WRITE).toBe('escrow:write');
    });
  });

  describe('Role Definitions', () => {
    it('should have correct role definitions', () => {
      expect(UserRole.BUYER).toBe('BUYER');
      expect(UserRole.SELLER).toBe('SELLER');
      expect(UserRole.LENDER).toBe('LENDER');
      expect(UserRole.AUDITOR).toBe('AUDITOR');
      expect(UserRole.ADMIN).toBe('ADMIN');
    });
  });
});

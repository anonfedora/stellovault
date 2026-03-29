-- Update Role enum values
ALTER TYPE "Role" RENAME TO "Role_old";
CREATE TYPE "Role" AS ENUM ('BUYER', 'SELLER', 'LENDER', 'AUDITOR', 'ADMIN');

-- Update existing users to use new role enum
ALTER TABLE "User" ALTER COLUMN "role" TYPE "Role" USING 'BUYER'::"Role";

-- Create RBAC tables
CREATE TABLE "Permission" (
    "id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "scope" TEXT NOT NULL,
    "description" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Permission_pkey" PRIMARY KEY ("id")
);

CREATE TABLE "roles" (
    "id" TEXT NOT NULL,
    "name" "Role" NOT NULL,
    "description" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "roles_pkey" PRIMARY KEY ("id")
);

CREATE TABLE "RolePermission" (
    "id" TEXT NOT NULL,
    "roleId" TEXT NOT NULL,
    "permissionId" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "RolePermission_pkey" PRIMARY KEY ("id")
);

CREATE TABLE "UserPermission" (
    "id" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "permissionId" TEXT NOT NULL,
    "grantedBy" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "UserPermission_pkey" PRIMARY KEY ("id")
);

-- Add RBAC relationships to User table
ALTER TABLE "User" ADD COLUMN "roleId" TEXT;
ALTER TABLE "User" ADD CONSTRAINT "User_roleId_fkey" FOREIGN KEY ("roleId") REFERENCES "roles"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- Create indexes
CREATE UNIQUE INDEX "Permission_scope_key" ON "Permission"("scope");
CREATE INDEX "Permission_scope_idx" ON "Permission"("scope");

CREATE UNIQUE INDEX "roles_name_key" ON "roles"("name");
CREATE INDEX "roles_name_idx" ON "roles"("name");

CREATE UNIQUE INDEX "RolePermission_roleId_permissionId_key" ON "RolePermission"("roleId", "permissionId");
CREATE INDEX "RolePermission_roleId_idx" ON "RolePermission"("roleId");
CREATE INDEX "RolePermission_permissionId_idx" ON "RolePermission"("permissionId");

CREATE UNIQUE INDEX "UserPermission_userId_permissionId_key" ON "UserPermission"("userId", "permissionId");
CREATE INDEX "UserPermission_userId_idx" ON "UserPermission"("userId");
CREATE INDEX "UserPermission_permissionId_idx" ON "UserPermission"("permissionId");

-- Insert default permissions
INSERT INTO "Permission" ("id", "name", "scope", "description") VALUES
('perm-1', 'Read Loans', 'loan:read', 'Permission to read loan information'),
('perm-2', 'Write Loans', 'loan:write', 'Permission to create and update loans'),
('perm-3', 'Delete Loans', 'loan:delete', 'Permission to delete loans'),
('perm-4', 'Approve Loans', 'loan:approve', 'Permission to approve loans'),
('perm-5', 'Read Escrows', 'escrow:read', 'Permission to read escrow information'),
('perm-6', 'Write Escrows', 'escrow:write', 'Permission to create and update escrows'),
('perm-7', 'Release Escrows', 'escrow:release', 'Permission to release escrow funds'),
('perm-8', 'Refund Escrows', 'escrow:refund', 'Permission to refund escrows'),
('perm-9', 'Read Payments', 'payment:read', 'Permission to read payment information'),
('perm-10', 'Write Payments', 'payment:write', 'Permission to create and update payments'),
('perm-11', 'Process Payments', 'payment:process', 'Permission to process payments'),
('perm-12', 'Read Users', 'user:read', 'Permission to read user information'),
('perm-13', 'Write Users', 'user:write', 'Permission to create and update users'),
('perm-14', 'Delete Users', 'user:delete', 'Permission to delete users'),
('perm-15', 'Read Audit', 'audit:read', 'Permission to read audit logs'),
('perm-16', 'Override Risk', 'risk:override', 'Permission to override risk scores'),
('perm-17', 'Admin All', 'admin:all', 'Full administrative access');

-- Insert default roles with permissions
INSERT INTO "roles" ("id", "name", "description") VALUES
('role-1', 'BUYER', 'Buyer role for creating escrows and managing payments'),
('role-2', 'SELLER', 'Seller role for managing escrows and receiving payments'),
('role-3', 'LENDER', 'Lender role for issuing and managing loans'),
('role-4', 'AUDITOR', 'Auditor role for read-only access to all data'),
('role-5', 'ADMIN', 'Administrator role with full system access');

-- Assign permissions to roles
INSERT INTO "RolePermission" ("id", "roleId", "permissionId") VALUES
-- Buyer permissions
('rp-1', 'role-1', 'perm-5'), -- escrow:read
('rp-2', 'role-1', 'perm-6'), -- escrow:write
('rp-3', 'role-1', 'perm-9'), -- payment:read
('rp-4', 'role-1', 'perm-10'), -- payment:write
('rp-5', 'role-1', 'perm-12'), -- user:read
('rp-6', 'role-1', 'perm-13'), -- user:write

-- Seller permissions
('rp-7', 'role-2', 'perm-5'), -- escrow:read
('rp-8', 'role-2', 'perm-7'), -- escrow:release
('rp-9', 'role-2', 'perm-9'), -- payment:read
('rp-10', 'role-2', 'perm-12'), -- user:read
('rp-11', 'role-2', 'perm-13'), -- user:write

-- Lender permissions
('rp-12', 'role-3', 'perm-1'), -- loan:read
('rp-13', 'role-3', 'perm-2'), -- loan:write
('rp-14', 'role-3', 'perm-4'), -- loan:approve
('rp-15', 'role-3', 'perm-9'), -- payment:read
('rp-16', 'role-3', 'perm-12'), -- user:read
('rp-17', 'role-3', 'perm-13'), -- user:write

-- Auditor permissions
('rp-18', 'role-4', 'perm-1'), -- loan:read
('rp-19', 'role-4', 'perm-5'), -- escrow:read
('rp-20', 'role-4', 'perm-9'), -- payment:read
('rp-21', 'role-4', 'perm-12'), -- user:read
('rp-22', 'role-4', 'perm-15'); -- audit:read

-- Admin permissions (all permissions)
('rp-23', 'role-5', 'perm-1'),
('rp-24', 'role-5', 'perm-2'),
('rp-25', 'role-5', 'perm-3'),
('rp-26', 'role-5', 'perm-4'),
('rp-27', 'role-5', 'perm-5'),
('rp-28', 'role-5', 'perm-6'),
('rp-29', 'role-5', 'perm-7'),
('rp-30', 'role-5', 'perm-8'),
('rp-31', 'role-5', 'perm-9'),
('rp-32', 'role-5', 'perm-10'),
('rp-33', 'role-5', 'perm-11'),
('rp-34', 'role-5', 'perm-12'),
('rp-35', 'role-5', 'perm-13'),
('rp-36', 'role-5', 'perm-14'),
('rp-37', 'role-5', 'perm-15'),
('rp-38', 'role-5', 'perm-16'),
('rp-39', 'role-5', 'perm-17');

-- Assign existing users to default BUYER role
UPDATE "User" SET "roleId" = 'role-1' WHERE "role" = 'BUYER';

-- Drop old role enum
DROP TYPE "Role_old";

import { Router } from "express";
import * as escrowController from "../controllers/escrow.controller";
import { authMiddleware } from "../middleware/auth.middleware";
import { checkPermission, checkOwnership, requireRole } from "../middleware/rbac.middleware";
import { ResourceType, UserRole } from "../types/rbac";

const router = Router();

// Apply authentication to all routes
router.use(authMiddleware);

// Create escrow - requires escrow:write permission (buyer role)
router.post(
  "/", 
  checkPermission(ResourceType.ESCROW, 'write'),
  requireRole(UserRole.BUYER),
  escrowController.createEscrow
);

// List escrows - requires escrow:read permission
router.get(
  "/", 
  checkPermission(ResourceType.ESCROW, 'read'),
  escrowController.listEscrows
);

// Get specific escrow - requires escrow:read and ownership check
router.get(
  "/:id", 
  checkPermission(ResourceType.ESCROW, 'read'),
  checkOwnership(ResourceType.ESCROW),
  escrowController.getEscrow
);

// Get escrow status - requires escrow:read permission
router.get(
  "/:id/status", 
  checkPermission(ResourceType.ESCROW, 'read'),
  escrowController.getEscrowStatus
);

// Webhook endpoint - public (no auth required for oracle callbacks)
router.post("/webhook", escrowController.webhookEscrowUpdate);

export default router;

import { Router } from "express";
import * as loanController from "../controllers/loan.controller";
import { authMiddleware } from "../middleware/auth.middleware";
import { auditLog } from "../middleware/audit.middleware";
import { checkPermission, checkOwnership, requireRole } from "../middleware/rbac.middleware";
import { ResourceType, UserRole } from "../types/rbac";

const router = Router();

router.use(authMiddleware);

// Create loan - requires lender:write permission
router.post(
  "/", 
  checkPermission(ResourceType.LOAN, 'write'),
  requireRole(UserRole.LENDER),
  auditLog("LOAN_CREATION"), 
  loanController.createLoan
);

// List loans - requires loan:read permission
router.get(
  "/", 
  checkPermission(ResourceType.LOAN, 'read'),
  loanController.listLoans
);

// Get specific loan - requires loan:read and ownership check
router.get(
  "/:id", 
  checkPermission(ResourceType.LOAN, 'read'),
  checkOwnership(ResourceType.LOAN),
  loanController.getLoan
);

// Create payment session - requires payment:write and loan ownership
router.post(
  "/:id/payments",
  checkPermission(ResourceType.PAYMENT, 'write'),
  checkOwnership(ResourceType.LOAN),
  auditLog("PAYMENT_SESSION_CREATED"),
  loanController.createPayment
);

// Record repayment - requires payment:write and loan ownership
router.post(
  "/repay",
  checkPermission(ResourceType.PAYMENT, 'write'),
  auditLog("LOAN_REPAYMENT"),
  loanController.recordRepayment
);

export default router;

import { Router } from "express";
import * as loanController from "../controllers/loan.controller";
import { authMiddleware } from "../middleware/auth.middleware";
import { auditLog } from "../middleware/audit.middleware";

const router = Router();

router.use(authMiddleware);

router.post("/", auditLog("LOAN_CREATION"), loanController.createLoan);
router.get("/", loanController.listLoans);
router.get("/:id", loanController.getLoan);
router.post("/repay", auditLog("LOAN_REPAYMENT"), loanController.recordRepayment);

export default router;

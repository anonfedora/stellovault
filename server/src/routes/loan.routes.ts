import { Router } from "express";
import * as loanController from "../controllers/loan.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

router.use(authMiddleware);

router.post("/", loanController.createLoan);
router.get("/", loanController.listLoans);
router.get("/:id", loanController.getLoan);
router.post("/repay", loanController.recordRepayment);

export default router;

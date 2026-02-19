import { Router } from "express";
import * as loanController from "../controllers/loan.controller";

const router = Router();

router.post("/", loanController.createLoan);
router.get("/", loanController.listLoans);
router.get("/:id", loanController.getLoan);
router.post("/repay", loanController.recordRepayment);

export default router;

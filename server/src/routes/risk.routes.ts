import { Router } from "express";
import * as riskController from "../controllers/risk.controller";
import { authMiddleware } from "../middleware/auth.middleware";
import { auditLog } from "../middleware/audit.middleware";

const router = Router();

router.get("/:wallet/history", riskController.getRiskHistory);
router.post("/:wallet/simulate", riskController.simulateRiskScore);
router.post("/:wallet/override", authMiddleware, auditLog("RISK_SCORE_OVERRIDE"), riskController.overrideRiskScore);
router.get("/:wallet", riskController.getRiskScore);

export default router;

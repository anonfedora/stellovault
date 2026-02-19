import { Router } from "express";
import * as riskController from "../controllers/risk.controller";

const router = Router();

router.get("/:wallet", riskController.getRiskScore);
router.get("/:wallet/history", riskController.getRiskHistory);
router.post("/:wallet/simulate", riskController.simulateRiskScore);

export default router;

import { Router } from "express";
import * as riskController from "../controllers/risk.controller";

const router = Router();

router.get("/:wallet/history", riskController.getRiskHistory);
router.post("/:wallet/simulate", riskController.simulateRiskScore);
router.get("/:wallet", riskController.getRiskScore);

export default router;

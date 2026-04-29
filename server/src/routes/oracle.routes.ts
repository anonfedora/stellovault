import { Router } from "express";
import * as oracleController from "../controllers/oracle.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

router.post("/", authMiddleware, oracleController.registerOracle);
router.post("/dispute", authMiddleware, oracleController.flagDispute);
router.get("/", oracleController.listOracles);
router.get("/metrics", oracleController.getOracleMetrics);
router.get("/network-status", oracleController.getOracleNetworkStatus);
router.get("/:address", oracleController.getOracle);
router.get("/:id/reputation", oracleController.getOracleReputation);
router.post("/:address/deactivate", authMiddleware, oracleController.deactivateOracle);
router.post("/rewards/distribute", authMiddleware, oracleController.distributeReward);
router.post("/stakes/slash", authMiddleware, oracleController.slashStake);
router.post("/disputes/:id/resolve", authMiddleware, oracleController.resolveDispute);
router.post("/threshold-signatures", authMiddleware, oracleController.createThresholdSignature);
router.post("/threshold-signatures/:id/sign", authMiddleware, oracleController.addThresholdSignature);
router.get("/threshold-signatures/:id", oracleController.getThresholdSignature);

export default router;

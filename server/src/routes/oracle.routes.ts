import { Router } from "express";
import * as oracleController from "../controllers/oracle.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

router.post("/", authMiddleware, oracleController.registerOracle);
router.get("/", oracleController.listOracles);
router.get("/metrics", oracleController.getOracleMetrics);
router.get("/:address", oracleController.getOracle);
router.post("/:address/deactivate", authMiddleware, oracleController.deactivateOracle);

export default router;

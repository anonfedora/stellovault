import { Router } from "express";
import * as oracleController from "../controllers/oracle.controller";

const router = Router();

router.post("/", oracleController.registerOracle);
router.get("/", oracleController.listOracles);
router.get("/metrics", oracleController.getOracleMetrics);
router.get("/:address", oracleController.getOracle);
router.post("/:address/deactivate", oracleController.deactivateOracle);

export default router;

import { Router } from "express";
import * as collateralController from "../controllers/collateral.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

router.get("/types", collateralController.getCollateralTypes);
router.post("/", authMiddleware, collateralController.tokenizeAsset);
router.get("/", collateralController.listCollateral);
router.get("/metadata/:hash", collateralController.getCollateralByMetadata);
router.get("/:id", collateralController.getCollateral);
router.get("/:id/metadata", collateralController.getCollateralMetadata);
router.get("/:id/ltv", collateralController.calculateLTV);
router.put("/:id/valuation", authMiddleware, collateralController.updateValuation);
router.post("/:id/verify", authMiddleware, collateralController.verifyCollateral);

export default router;

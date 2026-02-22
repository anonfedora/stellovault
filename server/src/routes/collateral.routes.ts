import { Router } from "express";
import * as collateralController from "../controllers/collateral.controller";

const router = Router();

router.post("/", collateralController.createCollateral);
router.get("/", collateralController.listCollateral);
router.get("/metadata/:hash", collateralController.getCollateralByMetadata);
router.get("/:id", collateralController.getCollateral);

export default router;

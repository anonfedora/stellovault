import { Router } from "express";
import * as analyticsController from "../controllers/analytics.controller";

const router = Router();

// Legacy endpoints
router.get("/", analyticsController.getPlatformStats);
router.get("/protocol", analyticsController.getProtocolAnalytics);

// New v1 analytics endpoints
router.get("/dashboard", analyticsController.getDashboardMetrics);
router.get("/volume", analyticsController.getVolumeReport);
router.get("/users", analyticsController.getUserAnalytics);
router.get("/performance", analyticsController.getPlatformPerformance);
router.post("/reports", analyticsController.generateCustomReport);
router.get("/compliance", analyticsController.getComplianceMetrics);

export default router;

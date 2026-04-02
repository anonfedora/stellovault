import { Router } from "express";
import * as analyticsController from "../controllers/analytics.controller";

const router = Router();

router.get("/", analyticsController.getPlatformStats);
router.get("/protocol", analyticsController.getProtocolAnalytics);

export default router;

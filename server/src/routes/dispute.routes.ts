import { Router } from "express";
import * as oracleController from "../controllers/oracle.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

router.post("/dispute", authMiddleware, oracleController.flagDispute);

export default router;

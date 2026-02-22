import { Router } from "express";
import * as oracleController from "../controllers/oracle.controller";

const router = Router();

router.post("/", oracleController.submitConfirmation);
router.get("/:escrowId", oracleController.getConfirmations);

export default router;

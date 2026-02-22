import { Router } from "express";
import * as escrowController from "../controllers/escrow.controller";

const router = Router();

router.post("/", escrowController.createEscrow);
router.get("/", escrowController.listEscrows);
router.get("/:id", escrowController.getEscrow);
router.post("/webhook", escrowController.webhookEscrowUpdate);

export default router;

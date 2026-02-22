import { Router } from "express";
import * as govController from "../controllers/governance.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

router.get("/proposals", govController.getProposals);
router.post("/proposals", authMiddleware, govController.createProposal);
router.get("/proposals/:id", govController.getProposal);
router.get("/proposals/:id/votes", govController.getProposalVotes);
router.post("/votes", authMiddleware, govController.submitVote);
router.get("/metrics", govController.getMetrics);
router.get("/parameters", govController.getParameters);
router.get("/audit", govController.getAuditLog);

export default router;

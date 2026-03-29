import { Router } from "express";
import * as govController from "../controllers/governance.controller";
import { authMiddleware } from "../middleware/auth.middleware";
import { auditLog } from "../middleware/audit.middleware";

const router = Router();

router.get("/proposals", govController.getProposals);
router.post("/proposals", authMiddleware, auditLog("GOVERNANCE_PROPOSAL_CREATION"), govController.createProposal);
router.get("/proposals/:id", govController.getProposal);
router.get("/proposals/:id/votes", govController.getProposalVotes);
router.post("/votes", authMiddleware, auditLog("GOVERNANCE_VOTE_SUBMISSION"), govController.submitVote);
router.get("/metrics", govController.getMetrics);
router.get("/parameters", govController.getParameters);
router.get("/audit", govController.getAuditLog);

export default router;

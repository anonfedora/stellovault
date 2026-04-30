import { Router } from "express";
import rateLimit from "express-rate-limit";
import { authMiddleware } from "../middleware/auth.middleware";
import * as webhookController from "../controllers/webhook.controller";

const router = Router();

const webhookMutationLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 30,
  standardHeaders: true,
  legacyHeaders: false,
});

const webhookTestLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 10,
  standardHeaders: true,
  legacyHeaders: false,
});

router.use(authMiddleware);

router.post("/", webhookMutationLimiter, webhookController.registerWebhook);
router.get("/", webhookController.listWebhooks);
router.put("/:id", webhookMutationLimiter, webhookController.updateWebhook);
router.delete("/:id", webhookMutationLimiter, webhookController.deleteWebhook);
router.post("/:id/test", webhookTestLimiter, webhookController.testWebhook);
router.get("/:id/logs", webhookController.getWebhookLogs);
router.get("/:id/metrics", webhookController.getWebhookMetrics);

export default router;

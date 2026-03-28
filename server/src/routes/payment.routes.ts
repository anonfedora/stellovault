// server/src/routes/payment.routes.ts
import { Router } from "express";
import { PaymentController } from "../controllers/payment.controller";

const router = Router();

router.get("/payments/:id/status", PaymentController.getStatus);
router.get("/payments/:id/stream", PaymentController.streamStatus);

export default router;

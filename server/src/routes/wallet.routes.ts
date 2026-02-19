import { Router } from "express";
import * as walletController from "../controllers/wallet.controller";
import { authMiddleware } from "../middleware/auth.middleware";

const router = Router();

// All wallet routes require authentication
router.use(authMiddleware);

router.get("/", walletController.listWallets);
router.post("/challenge", walletController.walletChallenge);
router.post("/", walletController.linkWallet);
router.delete("/:id", walletController.unlinkWallet);
router.put("/:id/primary", walletController.setPrimaryWallet);
router.patch("/:id", walletController.updateWallet);

export default router;

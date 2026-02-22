import { Router } from "express";
import * as userController from "../controllers/user.controller";

const router = Router();

router.get("/:id", userController.getUser);
router.post("/", userController.createUser);

export default router;

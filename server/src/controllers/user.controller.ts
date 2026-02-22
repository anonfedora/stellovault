import { Request, Response, NextFunction } from "express";
import userService from "../services/user.service";

/**
 * GET /api/users/:id
 * Returns the user profile including their wallets.
 * 404 when the user does not exist.
 */
export async function getUser(req: Request, res: Response, next: NextFunction) {
    try {
        const { id } = req.params;
        const user = await userService.getUserById(id);
        res.json({ success: true, data: user });
    } catch (err) { next(err); }
}

/**
 * POST /api/users
 * Idempotent user creation keyed on stellarAddress.
 * Returns the existing user if the address is already registered.
 */
export async function createUser(req: Request, res: Response, next: NextFunction) {
    try {
        const { stellarAddress } = req.body;
        const user = await userService.createUser(stellarAddress);
        res.status(201).json({ success: true, data: user });
    } catch (err) { next(err); }
}

export async function getAnalytics(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({
            success: true,
            data: {
                totalEscrows: 0,
                activeEscrows: 0,
                completedEscrows: 0,
                totalLoans: 0,
                activeLoans: 0,
                totalVolumeUSDC: "0",
                totalUsers: 0,
            },
        });
    } catch (err) { next(err); }
}

import { Request, Response, NextFunction } from "express";
import analyticsService from "../services/analytics.service";

export async function getPlatformStats(_req: Request, res: Response, next: NextFunction) {
    try {
        const stats = await analyticsService.getPlatformStats();
        // Acceptance requires flat JSON object with all stats
        res.json(stats);
    } catch (err) { next(err); }
}

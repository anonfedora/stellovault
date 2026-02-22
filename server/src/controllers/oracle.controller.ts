import { Request, Response, NextFunction } from "express";

export async function registerOracle(req: Request, res: Response, next: NextFunction) {
    try {
        res.status(201).json({ success: true, data: null });
    } catch (err) { next(err); }
}

export async function listOracles(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: [] });
    } catch (err) { next(err); }
}

export async function getOracle(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

export async function deactivateOracle(req: Request, res: Response, next: NextFunction) {
    try {
        res.status(204).send();
    } catch (err) { next(err); }
}

export async function submitConfirmation(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

export async function getConfirmations(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: [] });
    } catch (err) { next(err); }
}

export async function getOracleMetrics(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: { activeOracles: 0, confirmationRate: 0 } });
    } catch (err) { next(err); }
}

export async function flagDispute(req: Request, res: Response, next: NextFunction) {
    try {
        const { reason } = req.body;
        if (!reason) return res.status(400).json({ success: false, error: "Dispute reason is required" });
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

import { Request, Response, NextFunction } from "express";

export async function listCollateral(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: [] });
    } catch (err) { next(err); }
}

export async function createCollateral(req: Request, res: Response, next: NextFunction) {
    try {
        res.status(201).json({ success: true, data: null });
    } catch (err) { next(err); }
}

export async function getCollateral(req: Request, res: Response, next: NextFunction) {
    try {
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

export async function getCollateralByMetadata(req: Request, res: Response, next: NextFunction) {
    try {
        const { hash } = req.params;
        res.json({ success: true, data: null });
    } catch (err) { next(err); }
}

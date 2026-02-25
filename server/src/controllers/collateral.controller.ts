import { Request, Response, NextFunction } from "express";
import collateralService from "../services/collateral.service";

export async function createCollateral(req: Request, res: Response, next: NextFunction) {
    try {
        const result = await collateralService.createCollateral(req.body);
        res.status(201).json({ success: true, data: result });
    } catch (err) { next(err); }
}

export async function listCollateral(req: Request, res: Response, next: NextFunction) {
    try {
        const query = {
            escrowId: typeof req.query.escrowId === "string" ? req.query.escrowId : undefined,
            status: typeof req.query.status === "string" ? req.query.status : undefined,
            page: typeof req.query.page === "string" ? req.query.page : undefined,
            limit: typeof req.query.limit === "string" ? req.query.limit : undefined,
        };
        const result = await collateralService.listCollateral(query);
        res.json({ success: true, data: result });
    } catch (err) { next(err); }
}

export async function getCollateral(req: Request, res: Response, next: NextFunction) {
    try {
        const result = await collateralService.getCollateralById(req.params.id);
        res.json({ success: true, data: result });
    } catch (err) { next(err); }
}

export async function getCollateralByMetadata(req: Request, res: Response, next: NextFunction) {
    try {
        const result = await collateralService.getCollateralByMetadataHash(req.params.hash);
        res.json({ success: true, data: result });
    } catch (err) { next(err); }
}

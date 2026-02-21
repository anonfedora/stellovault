import { Request, Response, NextFunction } from "express";
import oracleService from "../services/oracle.service";

export async function registerOracle(req: Request, res: Response, next: NextFunction) {
    try {
        const { address } = req.body;
        const oracle = await oracleService.registerOracle({ address });
        res.status(201).json({ success: true, data: oracle });
    } catch (err) {
        next(err);
    }
}

export async function listOracles(req: Request, res: Response, next: NextFunction) {
    try {
        const isActive = req.query.isActive === "true" ? true : req.query.isActive === "false" ? false : undefined;
        const limit = req.query.limit ? parseInt(req.query.limit as string, 10) : 50;
        const offset = req.query.offset ? parseInt(req.query.offset as string, 10) : 0;

        const result = await oracleService.listOracles({ isActive, limit, offset });
        res.json({ success: true, data: result });
    } catch (err) {
        next(err);
    }
}

export async function getOracle(req: Request, res: Response, next: NextFunction) {
    try {
        const { address } = req.params;
        const oracle = await oracleService.getOracle(address);
        res.json({ success: true, data: oracle });
    } catch (err) {
        next(err);
    }
}

export async function deactivateOracle(req: Request, res: Response, next: NextFunction) {
    try {
        const { address } = req.params;
        await oracleService.deactivateOracle(address);
        res.status(204).send();
    } catch (err) {
        next(err);
    }
}

export async function submitConfirmation(req: Request, res: Response, next: NextFunction) {
    try {
        const { oracleAddress, escrowId, eventType, signature, payload, nonce } = req.body;
        const confirmation = await oracleService.confirmOracleEvent({
            oracleAddress,
            escrowId,
            eventType,
            signature,
            payload,
            nonce,
        });
        res.status(201).json({ success: true, data: confirmation });
    } catch (err) {
        next(err);
    }
}

export async function getConfirmations(req: Request, res: Response, next: NextFunction) {
    try {
        const { escrowId } = req.params;
        const limit = req.query.limit ? parseInt(req.query.limit as string, 10) : 50;
        const offset = req.query.offset ? parseInt(req.query.offset as string, 10) : 0;

        const result = await oracleService.getConfirmations({ escrowId, limit, offset });
        res.json({ success: true, data: result });
    } catch (err) {
        next(err);
    }
}

export async function getOracleMetrics(req: Request, res: Response, next: NextFunction) {
    try {
        const metrics = await oracleService.getOracleMetrics();
        res.json({ success: true, data: metrics });
    } catch (err) {
        next(err);
    }
}

export async function flagDispute(req: Request, res: Response, next: NextFunction) {
    try {
        const { escrowId, reason, disputerAddress } = req.body;
        const result = await oracleService.flagDispute({
            escrowId,
            reason,
            disputerAddress,
        });
        res.status(201).json({ success: true, data: result });
    } catch (err) {
        next(err);
    }
}

import { Request, Response, NextFunction } from "express";
import oracleService from "../services/oracle.service";
import { ValidationError } from "../config/errors";

function validateRequiredFields(fields: Record<string, unknown>, names: string[]): void {
    for (const name of names) {
        if (fields[name] === undefined || fields[name] === null || fields[name] === "") {
            throw new ValidationError(`Missing required field: ${name}`);
        }
    }
}

export async function registerOracle(req: Request, res: Response, next: NextFunction) {
    try {
        const { address } = req.body;
        const oracle = await oracleService.registerOracle({ address });
        res.status(201).json({ success: true, data: oracle });
    } catch (err) {
        next(err);
    }
}

function parsePaginationParam(value: unknown, defaultValue: number, min = 0, max = 1000): number {
    if (value === undefined || value === null || value === "") return defaultValue;
    const parsed = parseInt(String(value), 10);
    if (Number.isNaN(parsed)) return defaultValue;
    return Math.max(min, Math.min(max, parsed));
}

export async function listOracles(req: Request, res: Response, next: NextFunction) {
    try {
        const isActive = req.query.isActive === "true" ? true : req.query.isActive === "false" ? false : undefined;
        const limit = parsePaginationParam(req.query.limit, 50, 1, 100);
        const offset = parsePaginationParam(req.query.offset, 0, 0, 10000);

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
        validateRequiredFields(
            { oracleAddress, escrowId, eventType, signature, payload, nonce },
            ["oracleAddress", "escrowId", "eventType", "signature", "payload", "nonce"]
        );
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
        const limit = parsePaginationParam(req.query.limit, 50, 1, 100);
        const offset = parsePaginationParam(req.query.offset, 0, 0, 10000);

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
        validateRequiredFields(
            { escrowId, reason, disputerAddress },
            ["escrowId", "reason", "disputerAddress"]
        );
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

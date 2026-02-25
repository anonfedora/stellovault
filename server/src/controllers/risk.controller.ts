import { Request, Response, NextFunction } from "express";
import riskEngineService from "../services/risk-engine.service";
import { ValidationError } from "../config/errors";
import type { RiskSimulationScenario } from "../types/risk";

const STELLAR_ADDRESS_REGEX = /^G[A-Z2-7]{55}$|^C[A-Z2-7]{55}$/;

function parseDateParam(value: string | undefined, name: string): Date | undefined {
    if (value == null || value === "") return undefined;
    const d = new Date(value);
    if (Number.isNaN(d.getTime())) throw new ValidationError(`Invalid ${name}: not a valid date`);
    return d;
}

/**
 * GET /api/risk/:wallet
 * Compute current risk score for a Stellar wallet address.
 */
export async function getRiskScore(req: Request, res: Response, next: NextFunction) {
    try {
        const wallet = req.params.wallet;
        if (!wallet || !STELLAR_ADDRESS_REGEX.test(wallet)) {
            throw new ValidationError("Invalid wallet: must be a valid Stellar account address (G... or C...)");
        }
        const data = await riskEngineService.calculateRiskScore(wallet);
        res.json({ success: true, data });
    } catch (err) {
        next(err);
    }
}

/**
 * GET /api/risk/:wallet/history
 * Historical risk scores; query params: start_date=, end_date= (ISO date strings).
 */
export async function getRiskHistory(req: Request, res: Response, next: NextFunction) {
    try {
        const wallet = req.params.wallet;
        if (!wallet || !STELLAR_ADDRESS_REGEX.test(wallet)) {
            throw new ValidationError("Invalid wallet: must be a valid Stellar account address (G... or C...)");
        }
        const startDate = parseDateParam(req.query.start_date as string | undefined, "start_date");
        const endDate = parseDateParam(req.query.end_date as string | undefined, "end_date");
        const start = startDate ?? new Date(0);
        const end = endDate ?? new Date();
        if (start.getTime() > end.getTime()) {
            throw new ValidationError("start_date must be before or equal to end_date");
        }
        const data = await riskEngineService.getHistoricalScores(wallet, start, end);
        res.json({ success: true, data });
    } catch (err) {
        next(err);
    }
}

/**
 * POST /api/risk/:wallet/simulate
 * Simulate score impact of a hypothetical action. Does NOT persist the result.
 */
export async function simulateRiskScore(req: Request, res: Response, next: NextFunction) {
    try {
        const wallet = req.params.wallet;
        if (!wallet || !STELLAR_ADDRESS_REGEX.test(wallet)) {
            throw new ValidationError("Invalid wallet: must be a valid Stellar account address (G... or C...)");
        }
        const scenario: RiskSimulationScenario = req.body ?? {};
        const data = await riskEngineService.simulateScoreImpact(wallet, scenario);
        res.json({ success: true, data });
    } catch (err) {
        next(err);
    }
}

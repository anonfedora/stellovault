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

/**
 * POST /api/risk/:wallet/override
 * MANUALLY override risk score (ADMIN ONLY).
 */
export async function overrideRiskScore(req: Request, res: Response, next: NextFunction) {
    try {
        const wallet = req.params.wallet;
        const { score, reason } = req.body;
        
        if (!wallet || !STELLAR_ADDRESS_REGEX.test(wallet)) {
            throw new ValidationError("Invalid wallet address");
        }
        if (typeof score !== "number" || score < 0 || score > 100) {
            throw new ValidationError("Invalid score: must be between 0 and 100");
        }
        if (!reason) {
            throw new ValidationError("Reason is required for manual override");
        }

        // In a real implementation, you'd call a service to persist this.
        // For now, we simulate success for the purpose of demonstrating audit logging.
        res.json({ 
            success: true, 
            message: "Risk score successfully overridden",
            data: { wallet, newScore: score, reason }
        });
    } catch (err) {
        next(err);
    }
}

import { prisma } from "./database.service";
import blockchainService from "./blockchain.service";
import type {
    RiskScoreResponse,
    RiskGrade,
    RiskScoreComponents,
    RiskSimulationScenario,
    RiskSimulateResponse,
} from "../types/risk";

/** Loan-like record from DB (avoids depending on generated Prisma types at test time). */
type LoanRecord = { status: string; amount: unknown; collateralAmt: unknown };
/** RiskScore row from DB. */
type RiskScoreRow = { wallet: string; score: number; grade: string; components: unknown; computedAt: Date };

const COMPONENT_MAX = 250; // each component 0–250, total 0–1000
function scoreToGrade(score: number): RiskGrade {
    if (score >= 800) return "A";
    if (score >= 600) return "B";
    if (score >= 400) return "C";
    if (score >= 200) return "D";
    return "F";
}

/** Normalize a 0–1 factor to 0–COMPONENT_MAX. */
function toComponent(factor: number): number {
    return Math.round(Math.max(0, Math.min(1, factor)) * COMPONENT_MAX);
}

/**
 * Fetch Horizon transaction history and compute transaction-history component (volume, frequency, age).
 * Returns 0–250.
 */
async function computeTransactionHistoryComponent(wallet: string): Promise<number> {
    let records: Array<{ created_at: string }> = [];
    try {
        const txs = await blockchainService.getTransactionHistory(wallet, 200);
        records = txs as Array<{ created_at: string }>;
    } catch {
        return 0;
    }
    if (records.length === 0) return 0;
    const now = Date.now() / 1000;
    const oldest = Math.min(...records.map((r) => new Date(r.created_at).getTime() / 1000));
    const ageYears = (now - oldest) / (365.25 * 24 * 3600);
    const frequency = records.length; // more txs = more activity
    const volumeScore = Math.min(1, frequency / 100); // cap at 100 txs for “full” volume
    const ageScore = Math.min(1, ageYears / 2); // 2+ years = full age score
    const factor = 0.5 * volumeScore + 0.5 * ageScore;
    return toComponent(factor);
}

/**
 * Compute repayment-record component from DB loans (on-time vs defaulted).
 * Returns 0–250.
 */
async function computeRepaymentRecordComponent(wallet: string): Promise<number> {
    const user = await prisma.user.findUnique({ where: { stellarAddress: wallet } });
    if (!user) return COMPONENT_MAX; // no loans = neutral (full score)
    const loans = await prisma.loan.findMany({
        where: { borrowerId: user.id, status: { in: ["REPAID", "DEFAULTED"] } },
    });
    const repaid = loans.filter((l: LoanRecord) => l.status === "REPAID").length;
    const defaulted = loans.filter((l: LoanRecord) => l.status === "DEFAULTED").length;
    const total = repaid + defaulted;
    if (total === 0) return COMPONENT_MAX;
    const factor = repaid / total;
    return toComponent(factor);
}

/**
 * Compute collateral-coverage component from DB loans (collateralAmt / amount).
 * Returns 0–250. Ratio >= 1.5 => 250, 1.0 => ~167, 0 => 0.
 */
async function computeCollateralCoverageComponent(wallet: string): Promise<number> {
    const user = await prisma.user.findUnique({ where: { stellarAddress: wallet } });
    if (!user) return COMPONENT_MAX;
    const loans = await prisma.loan.findMany({
        where: { borrowerId: user.id },
    });
    if (loans.length === 0) return COMPONENT_MAX;
    let sumRatio = 0;
    let count = 0;
    for (const l of loans) {
        const amt = Number(l.amount);
        const coll = Number(l.collateralAmt);
        if (amt > 0) {
            sumRatio += coll / amt;
            count++;
        }
    }
    if (count === 0) return COMPONENT_MAX;
    const avgRatio = sumRatio / count;
    const factor = Math.min(1, avgRatio / 1.5); // 1.5+ ratio = full score
    return toComponent(factor);
}

/**
 * Escrow dispute history. No Dispute table in schema yet — use 0 disputes = full score.
 * Returns 0–250.
 */
async function computeDisputeHistoryComponent(wallet: string): Promise<number> {
    // TODO: when Escrow/Dispute table exists, count disputes for wallet and reduce score
    return COMPONENT_MAX;
}

/**
 * Compute all components and total score (0–1000). Does not persist.
 * Optional overrides apply hypothetical scenario for simulation.
 */
async function computeScore(
    wallet: string,
    scenarioOverrides?: RiskSimulationScenario
): Promise<{ score: number; grade: RiskGrade; components: RiskScoreComponents }> {
    let transactionHistory = await computeTransactionHistoryComponent(wallet);
    let repaymentRecord = await computeRepaymentRecordComponent(wallet);
    let collateralCoverage = await computeCollateralCoverageComponent(wallet);
    let disputeHistory = await computeDisputeHistoryComponent(wallet);

    if (scenarioOverrides) {
        if (scenarioOverrides.addDefault) {
            const user = await prisma.user.findUnique({ where: { stellarAddress: wallet } });
            const loans = user
                ? await prisma.loan.findMany({
                      where: { borrowerId: user.id, status: { in: ["REPAID", "DEFAULTED"] } },
                  })
                : [];
            const total = loans.length + 1;
            const defaulted = loans.filter((l: LoanRecord) => l.status === "DEFAULTED").length + 1;
            repaymentRecord = toComponent(1 - defaulted / total);
        }
        if (scenarioOverrides.addRepayment) {
            const user = await prisma.user.findUnique({ where: { stellarAddress: wallet } });
            const loans = user
                ? await prisma.loan.findMany({
                      where: { borrowerId: user.id, status: { in: ["REPAID", "DEFAULTED"] } },
                  })
                : [];
            const total = loans.length + 1;
            const repaid = loans.filter((l: LoanRecord) => l.status === "REPAID").length + 1;
            repaymentRecord = toComponent(repaid / total);
        }
        if (scenarioOverrides.addLoan?.amount != null && scenarioOverrides.addLoan?.collateralAmt != null) {
            const user = await prisma.user.findUnique({ where: { stellarAddress: wallet } });
            const loans = user ? await prisma.loan.findMany({ where: { borrowerId: user.id } }) : [];
            const virtualLoans = [
                ...loans.map((l: LoanRecord) => ({ amount: Number(l.amount), collateralAmt: Number(l.collateralAmt) })),
                {
                    amount: scenarioOverrides.addLoan.amount,
                    collateralAmt: scenarioOverrides.addLoan.collateralAmt,
                },
            ];
            const count = virtualLoans.length;
            const sumRatio = virtualLoans.reduce(
                (s: number, l: { amount: number; collateralAmt: number }) =>
                    s + (l.amount > 0 ? l.collateralAmt / l.amount : 0),
                0
            );
            const avgRatio = count ? sumRatio / count : 0;
            collateralCoverage = toComponent(Math.min(1, avgRatio / 1.5));
        }
        if (scenarioOverrides.collateralRatioChange != null) {
            const user = await prisma.user.findUnique({ where: { stellarAddress: wallet } });
            const loans = user ? await prisma.loan.findMany({ where: { borrowerId: user.id } }) : [];
            const currentAvg =
                loans.length > 0
                    ? loans.reduce(
                          (s: number, l: LoanRecord) =>
                              s + (Number(l.amount) > 0 ? Number(l.collateralAmt) / Number(l.amount) : 0),
                          0
                      ) /
                      loans.length
                    : 0;
            const newRatio = currentAvg * scenarioOverrides.collateralRatioChange;
            collateralCoverage = toComponent(Math.min(1, newRatio / 1.5));
        }
        if (scenarioOverrides.addDispute) {
            disputeHistory = toComponent(0); // one dispute => min component for now
        }
    }

    const components: RiskScoreComponents = {
        transactionHistory,
        repaymentRecord,
        collateralCoverage,
        disputeHistory,
    };
    const score = Math.round(
        transactionHistory + repaymentRecord + collateralCoverage + disputeHistory
    );
    const clampedScore = Math.max(0, Math.min(1000, score));
    const grade = scoreToGrade(clampedScore);
    return { score: clampedScore, grade, components };
}

export class RiskEngineService {
    /**
     * Compute current risk score for a wallet, persist snapshot, return response.
     */
    async calculateRiskScore(wallet: string): Promise<RiskScoreResponse> {
        const { score, grade, components } = await computeScore(wallet);
        const computedAt = new Date();
        await prisma.riskScore.create({
            data: {
                wallet,
                score,
                grade,
                components: components as object,
                computedAt,
            },
        });
        return {
            wallet,
            score,
            grade,
            components,
            computedAt,
        };
    }

    /**
     * Historical risk scores for wallet in date range.
     */
    async getHistoricalScores(
        wallet: string,
        startDate: Date,
        endDate: Date
    ): Promise<RiskScoreResponse[]> {
        const rows = await prisma.riskScore.findMany({
            where: {
                wallet,
                computedAt: { gte: startDate, lte: endDate },
            },
            orderBy: { computedAt: "asc" },
        });
        return rows.map((r: RiskScoreRow) => ({
            wallet: r.wallet,
            score: r.score,
            grade: r.grade as RiskGrade,
            components: r.components as RiskScoreComponents,
            computedAt: r.computedAt,
        }));
    }

    /**
     * Simulate score impact of a hypothetical scenario. Does NOT persist.
     */
    async simulateScoreImpact(wallet: string, scenario: RiskSimulationScenario): Promise<RiskSimulateResponse> {
        const current = await computeScore(wallet);
        const projected = await computeScore(wallet, scenario);
        return {
            currentScore: current.score,
            projectedScore: projected.score,
            delta: projected.score - current.score,
        };
    }
}

export default new RiskEngineService();

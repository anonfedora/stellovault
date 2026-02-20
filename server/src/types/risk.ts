/**
 * Risk scoring types for Stellar wallet risk API.
 */

export type RiskGrade = "A" | "B" | "C" | "D" | "F";

export type RiskScoreComponents = {
    transactionHistory: number;
    repaymentRecord: number;
    collateralCoverage: number;
    disputeHistory: number;
};

export type RiskScoreResponse = {
    wallet: string;
    score: number; // 0–1000
    grade: RiskGrade;
    components: RiskScoreComponents;
    computedAt: Date;
};

/** Hypothetical scenario for simulate endpoint (e.g. new loan, default, collateral change). */
export type RiskSimulationScenario = {
    /** Add a hypothetical loan: { amount, collateralAmt } */
    addLoan?: { amount: number; collateralAmt: number };
    /** Simulate one more defaulted loan */
    addDefault?: boolean;
    /** Simulate one more repaid loan */
    addRepayment?: boolean;
    /** Simulate change in collateral coverage ratio (e.g. 1.5 = 150%) */
    collateralRatioChange?: number;
    /** Simulate one more escrow dispute */
    addDispute?: boolean;
};

export type RiskSimulateResponse = {
    currentScore: number;
    projectedScore: number;
    delta: number;
};

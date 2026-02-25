/**
 * Unit tests for the risk engine. Mocks DB and Horizon so no real database is needed.
 * Run: npm test
 */

jest.mock("./database.service", () => ({
  prisma: {
    user: { findUnique: jest.fn() },
    loan: { findMany: jest.fn() },
    riskScore: { create: jest.fn().mockResolvedValue(undefined), findMany: jest.fn() },
  },
}));

jest.mock("./blockchain.service", () => ({
  __esModule: true,
  default: { getTransactionHistory: jest.fn().mockResolvedValue([]) },
}));

import { prisma } from "./database.service";
import blockchainService from "./blockchain.service";
import riskEngineService from "./risk-engine.service";

const WALLET = "GB6NVEN5HSUBKMYCE5ZOWSK5K23TBWRUQLZY3KNMXUZ3AQ2ESC4MY4AQ";

beforeEach(() => {
  jest.clearAllMocks();
  (prisma.user.findUnique as jest.Mock).mockResolvedValue(null);
  (prisma.loan.findMany as jest.Mock).mockResolvedValue([]);
  (prisma.riskScore.findMany as jest.Mock).mockResolvedValue([]);
  (prisma.riskScore.create as jest.Mock).mockResolvedValue(undefined);
  (blockchainService.getTransactionHistory as jest.Mock).mockResolvedValue([]);
});

describe("RiskEngineService", () => {
  describe("calculateRiskScore", () => {
    it("returns score between 0 and 1000", async () => {
      const result = await riskEngineService.calculateRiskScore(WALLET);
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1000);
    });

    it("returns grade A, B, C, D, or F", async () => {
      const result = await riskEngineService.calculateRiskScore(WALLET);
      expect(["A", "B", "C", "D", "F"]).toContain(result.grade);
    });

    it("returns all four components", async () => {
      const result = await riskEngineService.calculateRiskScore(WALLET);
      expect(result).toMatchObject({
        wallet: WALLET,
        components: {
          transactionHistory: expect.any(Number),
          repaymentRecord: expect.any(Number),
          collateralCoverage: expect.any(Number),
          disputeHistory: expect.any(Number),
        },
      });
      expect(result.computedAt).toBeInstanceOf(Date);
    });

    it("persists a RiskScore snapshot (calls prisma.riskScore.create)", async () => {
      await riskEngineService.calculateRiskScore(WALLET);
      expect(prisma.riskScore.create).toHaveBeenCalledTimes(1);
      expect(prisma.riskScore.create).toHaveBeenCalledWith(
        expect.objectContaining({
          data: expect.objectContaining({
            wallet: WALLET,
            score: expect.any(Number),
            grade: expect.any(String),
            components: expect.any(Object),
          }),
        })
      );
    });
  });

  describe("getHistoricalScores", () => {
    it("filters by date range (calls findMany with gte/lte)", async () => {
      const start = new Date("2025-01-01");
      const end = new Date("2025-12-31");
      (prisma.riskScore.findMany as jest.Mock).mockResolvedValue([
        {
          id: "rs1",
          wallet: WALLET,
          score: 500,
          grade: "C",
          components: { transactionHistory: 125, repaymentRecord: 125, collateralCoverage: 125, disputeHistory: 125 },
          computedAt: new Date("2025-06-01"),
        },
      ]);
      const result = await riskEngineService.getHistoricalScores(WALLET, start, end);
      expect(prisma.riskScore.findMany).toHaveBeenCalledWith({
        where: {
          wallet: WALLET,
          computedAt: { gte: start, lte: end },
        },
        orderBy: { computedAt: "asc" },
      });
      expect(result).toHaveLength(1);
      expect(result[0].score).toBe(500);
      expect(result[0].grade).toBe("C");
    });

    it("returns empty array when no history in range", async () => {
      (prisma.riskScore.findMany as jest.Mock).mockResolvedValue([]);
      const result = await riskEngineService.getHistoricalScores(
        WALLET,
        new Date("2025-01-01"),
        new Date("2025-12-31")
      );
      expect(result).toEqual([]);
    });
  });

  describe("simulateScoreImpact", () => {
    it("returns currentScore, projectedScore, and delta", async () => {
      const result = await riskEngineService.simulateScoreImpact(WALLET, {});
      expect(result).toHaveProperty("currentScore");
      expect(result).toHaveProperty("projectedScore");
      expect(result).toHaveProperty("delta");
      expect(typeof result.currentScore).toBe("number");
      expect(typeof result.projectedScore).toBe("number");
      expect(result.delta).toBe(result.projectedScore - result.currentScore);
    });

    it("does NOT persist the simulated score (never calls riskScore.create)", async () => {
      await riskEngineService.simulateScoreImpact(WALLET, { addDefault: true });
      expect(prisma.riskScore.create).not.toHaveBeenCalled();
    });

    it("projected score differs from current when scenario is applied", async () => {
      (prisma.user.findUnique as jest.Mock).mockResolvedValue({ id: "u1", stellarAddress: WALLET });
      (prisma.loan.findMany as jest.Mock).mockResolvedValue([]);
      const resultEmpty = await riskEngineService.simulateScoreImpact(WALLET, {});
      expect(resultEmpty.delta).toBe(0);
      const resultWithDefault = await riskEngineService.simulateScoreImpact(WALLET, { addDefault: true });
      expect(resultWithDefault.projectedScore).toBeLessThanOrEqual(resultWithDefault.currentScore);
      expect(resultWithDefault.delta).toBeLessThanOrEqual(0);
    });
  });

  describe("acceptance criteria", () => {
    it("score is always between 0 and 1000", async () => {
      (prisma.user.findUnique as jest.Mock).mockResolvedValue({ id: "u1", stellarAddress: WALLET });
      (prisma.loan.findMany as jest.Mock).mockResolvedValue([
        { status: "DEFAULTED", amount: 100, collateralAmt: 0 },
        { status: "REPAID", amount: 100, collateralAmt: 150 },
      ]);
      const result = await riskEngineService.calculateRiskScore(WALLET);
      expect(result.score).toBeGreaterThanOrEqual(0);
      expect(result.score).toBeLessThanOrEqual(1000);
    });
  });
});

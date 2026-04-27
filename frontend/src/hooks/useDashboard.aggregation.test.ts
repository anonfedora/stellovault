import assert from "node:assert/strict";
import { LoanStatus, TransactionStatus } from "../types";
import type { Loan } from "../types";
import {
  aggregatePortfolio,
  applyRealtimeEvent,
  buildActivityFeed,
  buildMetricsSeries,
  type DashboardData,
  type DashboardRealtimeEvent,
} from "./useDashboard";

interface TestCase {
  name: string;
  run: () => void | Promise<void>;
}

const sampleLoans: Loan[] = [
  {
    id: "loan-a",
    borrower: "GA",
    collateralTokenId: "ct-a",
    collateralAssetType: "INVOICE",
    collateralValue: 20_000,
    principal: 10_000,
    interestRate: 10,
    termMonths: 12,
    status: LoanStatus.ACTIVE,
    repayments: [
      {
        id: "rep-a-1",
        loanId: "loan-a",
        amount: 4_000,
        paidAt: new Date("2026-02-01T00:00:00Z"),
        txHash: "txhasha1",
        status: TransactionStatus.COMPLETED,
      },
    ],
    createdAt: new Date("2026-01-01T00:00:00Z"),
    maturityDate: new Date("2027-01-01T00:00:00Z"),
  },
  {
    id: "loan-b",
    borrower: "GB",
    collateralTokenId: "ct-b",
    collateralAssetType: "COMMODITY",
    collateralValue: 5_000,
    principal: 2_000,
    interestRate: 0,
    termMonths: 6,
    status: LoanStatus.PENDING,
    repayments: [],
    createdAt: new Date("2026-03-15T00:00:00Z"),
    maturityDate: new Date("2026-09-15T00:00:00Z"),
  },
  {
    id: "loan-c",
    borrower: "GC",
    collateralTokenId: "ct-c",
    collateralAssetType: "RECEIVABLE",
    collateralValue: 8_000,
    principal: 4_000,
    interestRate: 5,
    termMonths: 6,
    status: LoanStatus.DEFAULTED,
    repayments: [],
    createdAt: new Date("2025-09-01T00:00:00Z"),
    maturityDate: new Date("2026-03-01T00:00:00Z"),
  },
];

export const tests: TestCase[] = [
  {
    name: "aggregatePortfolio sums collateral, principal, and outstanding correctly",
    run: () => {
      const result = aggregatePortfolio(sampleLoans);
      // Collateral = 20000 + 5000 + 8000
      assert.equal(result.totalCollateralValue, 33_000);
      // Principal = 10000 + 2000 + 4000
      assert.equal(result.totalPrincipal, 16_000);
      assert.equal(result.totalRepaid, 4_000);
      // Owed = 11000 + 2000 + 4200; outstanding = 17200 - 4000
      assert.equal(result.totalOutstanding, 13_200);
      assert.equal(result.activeLoanCount, 1);
      assert.equal(result.pendingLoanCount, 1);
      assert.equal(result.defaultedLoanCount, 1);
    },
  },
  {
    name: "aggregatePortfolio handles empty input safely",
    run: () => {
      const result = aggregatePortfolio([]);
      assert.equal(result.totalCollateralValue, 0);
      assert.equal(result.totalOutstanding, 0);
      assert.equal(result.utilization, 0);
    },
  },
  {
    name: "buildActivityFeed orders newest first and includes repayments",
    run: () => {
      const feed = buildActivityFeed(sampleLoans);
      assert.ok(feed.length >= 4);
      // First item should be the most recent loan creation (2026-03-15).
      assert.equal(feed[0].timestamp, "2026-03-15T00:00:00.000Z");
      const repaymentItems = feed.filter((i) => i.category === "repayment");
      assert.equal(repaymentItems.length, 1);
      assert.equal(repaymentItems[0].status, "success");
    },
  },
  {
    name: "buildMetricsSeries produces non-decreasing repaid values",
    run: () => {
      const series = buildMetricsSeries(sampleLoans);
      assert.ok(series.length > 0);
      for (let i = 1; i < series.length; i += 1) {
        assert.ok(
          series[i].repaid >= series[i - 1].repaid,
          `repaid decreased at index ${i}`,
        );
      }
      // Outstanding can only be non-negative.
      for (const point of series) {
        assert.ok(point.outstanding >= 0);
      }
    },
  },
  {
    name: "applyRealtimeEvent prepends a notification with severity by type",
    run: () => {
      const data: DashboardData = {
        portfolio: aggregatePortfolio(sampleLoans),
        loans: sampleLoans,
        activity: [],
        notifications: [],
        metrics: [],
      };
      const event: DashboardRealtimeEvent = {
        type: "LoanDefaulted",
        occurredAt: new Date("2026-04-25T12:00:00Z").toISOString(),
        payload: { loanId: "loan-c" },
      };
      const next = applyRealtimeEvent(data, event);
      assert.equal(next.notifications.length, 1);
      assert.equal(next.notifications[0].severity, "critical");
      assert.match(next.notifications[0].body, /loan-c/);
    },
  },
];

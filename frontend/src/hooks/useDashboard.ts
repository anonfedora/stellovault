"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Loan } from "@/types";
import { LoanStatus, TransactionStatus } from "@/types";
import { useTransactionStatus } from "@/hooks/useTransactionStatus";
import { computeReconnectDelay } from "@/hooks/useWebSocket";

export type DashboardEventType =
  | "LoanCreated"
  | "LoanRepaid"
  | "LoanDefaulted"
  | "EscrowReleased"
  | "RiskScoreUpdated"
  | "NotificationCreated";

export interface DashboardRealtimeEvent {
  type: DashboardEventType;
  occurredAt: string;
  payload?: Record<string, unknown>;
}

export interface PortfolioSnapshot {
  totalCollateralValue: number;
  totalPrincipal: number;
  totalOutstanding: number;
  totalRepaid: number;
  activeLoanCount: number;
  pendingLoanCount: number;
  defaultedLoanCount: number;
  netPosition: number;
  utilization: number;
}

export interface ActivityItem {
  id: string;
  title: string;
  description: string;
  timestamp: string;
  category: "loan" | "repayment" | "escrow" | "system";
  amount?: number;
  status?: "success" | "pending" | "failed" | "info";
}

export interface DashboardNotification {
  id: string;
  title: string;
  body: string;
  createdAt: string;
  severity: "info" | "warning" | "critical" | "success";
  read: boolean;
}

export interface MetricsPoint {
  date: string;
  outstanding: number;
  repaid: number;
}

export interface DashboardData {
  portfolio: PortfolioSnapshot;
  loans: Loan[];
  activity: ActivityItem[];
  notifications: DashboardNotification[];
  metrics: MetricsPoint[];
}

interface UseDashboardOptions {
  walletAddress?: string | null;
  enableRealtime?: boolean;
  refreshIntervalMs?: number;
}

interface UseDashboardReturn extends DashboardData {
  loading: boolean;
  error: string | null;
  connected: boolean;
  lastUpdated: Date | null;
  refresh: () => Promise<void>;
  markNotificationRead: (id: string) => void;
  clearNotifications: () => void;
}

const MOCK_LOANS: Loan[] = [
  {
    id: "loan-1",
    borrower: "GBXYZ4KITP4KVFJN4OGR6MXBGHOBRCGIXAQ7ALIEQVRSWZLUQBIHIG2K",
    collateralTokenId: "ct-001",
    collateralAssetType: "INVOICE",
    collateralValue: 50000,
    principal: 25000,
    interestRate: 8.5,
    termMonths: 12,
    status: LoanStatus.ACTIVE,
    repayments: [
      {
        id: "rep-1",
        loanId: "loan-1",
        amount: 5200,
        paidAt: new Date("2026-01-15"),
        txHash: "abc123",
        status: TransactionStatus.COMPLETED,
      },
      {
        id: "rep-2",
        loanId: "loan-1",
        amount: 5200,
        paidAt: new Date("2026-02-15"),
        txHash: "def456",
        status: TransactionStatus.COMPLETED,
      },
    ],
    createdAt: new Date("2025-11-01"),
    maturityDate: new Date("2026-11-01"),
  },
  {
    id: "loan-2",
    borrower: "GCABC5MJUQ8LXFGV5PGRAYHNQWFS2HP5EXLSM6MWQKVB5ZDLTRCKNJ7",
    collateralTokenId: "ct-002",
    collateralAssetType: "COMMODITY",
    collateralValue: 75000,
    principal: 40000,
    interestRate: 7.0,
    termMonths: 6,
    status: LoanStatus.ACTIVE,
    repayments: [
      {
        id: "rep-3",
        loanId: "loan-2",
        amount: 3100,
        paidAt: new Date("2026-03-10"),
        txHash: "ghi789",
        status: TransactionStatus.COMPLETED,
      },
    ],
    createdAt: new Date("2025-12-15"),
    maturityDate: new Date("2026-06-15"),
  },
  {
    id: "loan-3",
    borrower: "GDDEF6NKUP9EYGHW6QHSBIYZNE3XUFRSSZ7MWQLNB6CXDMQTRSXON8P",
    collateralTokenId: "ct-003",
    collateralAssetType: "RECEIVABLE",
    collateralValue: 30000,
    principal: 15000,
    interestRate: 9.0,
    termMonths: 3,
    status: LoanStatus.PENDING,
    repayments: [],
    createdAt: new Date("2026-04-01"),
    maturityDate: new Date("2026-07-01"),
  },
];

const INITIAL_NOTIFICATIONS: DashboardNotification[] = [
  {
    id: "n-1",
    title: "Repayment received",
    body: "Loan loan-1 received a 5,200 USDC repayment.",
    createdAt: new Date("2026-04-22T10:00:00Z").toISOString(),
    severity: "success",
    read: false,
  },
  {
    id: "n-2",
    title: "Escrow awaiting release",
    body: "Escrow esc-009 is awaiting oracle confirmation.",
    createdAt: new Date("2026-04-23T08:30:00Z").toISOString(),
    severity: "warning",
    read: false,
  },
  {
    id: "n-3",
    title: "Risk score updated",
    body: "Your aggregate risk score improved to A.",
    createdAt: new Date("2026-04-24T15:12:00Z").toISOString(),
    severity: "info",
    read: true,
  },
];

// ── Pure helpers (exported for unit tests) ─────────────────────────────────

export function aggregatePortfolio(loans: Loan[]): PortfolioSnapshot {
  let totalCollateralValue = 0;
  let totalPrincipal = 0;
  let totalRepaid = 0;
  let totalOwed = 0;
  let activeLoanCount = 0;
  let pendingLoanCount = 0;
  let defaultedLoanCount = 0;

  for (const loan of loans) {
    totalCollateralValue += loan.collateralValue;
    totalPrincipal += loan.principal;
    const owed = loan.principal * (1 + loan.interestRate / 100);
    totalOwed += owed;

    const repaid = loan.repayments.reduce((sum, r) => sum + r.amount, 0);
    totalRepaid += repaid;

    if (loan.status === LoanStatus.ACTIVE) activeLoanCount += 1;
    else if (loan.status === LoanStatus.PENDING) pendingLoanCount += 1;
    else if (loan.status === LoanStatus.DEFAULTED) defaultedLoanCount += 1;
  }

  const totalOutstanding = Math.max(0, totalOwed - totalRepaid);
  const utilization =
    totalCollateralValue === 0 ? 0 : totalPrincipal / totalCollateralValue;

  return {
    totalCollateralValue,
    totalPrincipal,
    totalOutstanding,
    totalRepaid,
    activeLoanCount,
    pendingLoanCount,
    defaultedLoanCount,
    netPosition: totalCollateralValue - totalOutstanding,
    utilization,
  };
}

export function buildActivityFeed(loans: Loan[]): ActivityItem[] {
  const items: ActivityItem[] = [];

  for (const loan of loans) {
    items.push({
      id: `loan-created-${loan.id}`,
      title: `New ${loan.collateralAssetType.toLowerCase()} loan`,
      description: `Loan ${loan.id} originated for $${loan.principal.toLocaleString()}.`,
      timestamp: loan.createdAt.toISOString(),
      category: "loan",
      amount: loan.principal,
      status: "info",
    });

    for (const repayment of loan.repayments) {
      items.push({
        id: `repayment-${repayment.id}`,
        title: `Repayment on ${loan.id}`,
        description: `Received $${repayment.amount.toLocaleString()} (tx ${repayment.txHash.slice(0, 8)}…).`,
        timestamp: repayment.paidAt.toISOString(),
        category: "repayment",
        amount: repayment.amount,
        status:
          repayment.status === TransactionStatus.COMPLETED
            ? "success"
            : repayment.status === TransactionStatus.FAILED
              ? "failed"
              : "pending",
      });
    }
  }

  items.sort((a, b) => (a.timestamp < b.timestamp ? 1 : -1));
  return items;
}

export function buildMetricsSeries(loans: Loan[]): MetricsPoint[] {
  if (loans.length === 0) return [];

  const events: { date: Date; outstanding: number; repaid: number }[] = [];

  for (const loan of loans) {
    const owed = loan.principal * (1 + loan.interestRate / 100);
    events.push({ date: loan.createdAt, outstanding: owed, repaid: 0 });
    for (const repayment of loan.repayments) {
      events.push({
        date: repayment.paidAt,
        outstanding: -repayment.amount,
        repaid: repayment.amount,
      });
    }
  }

  events.sort((a, b) => a.date.getTime() - b.date.getTime());

  const series: MetricsPoint[] = [];
  let outstanding = 0;
  let repaid = 0;
  for (const ev of events) {
    outstanding = Math.max(0, outstanding + ev.outstanding);
    repaid += ev.repaid;
    series.push({
      date: ev.date.toISOString().slice(0, 10),
      outstanding: Math.round(outstanding),
      repaid: Math.round(repaid),
    });
  }

  // Collapse points sharing the same date keeping the latest values.
  const dedup = new Map<string, MetricsPoint>();
  for (const point of series) dedup.set(point.date, point);
  return Array.from(dedup.values());
}

export function applyRealtimeEvent(
  data: DashboardData,
  event: DashboardRealtimeEvent,
): DashboardData {
  const newNotification: DashboardNotification = {
    id: `${event.type}-${event.occurredAt}`,
    title: notificationTitleFor(event.type),
    body: notificationBodyFor(event),
    createdAt: event.occurredAt,
    severity:
      event.type === "LoanDefaulted"
        ? "critical"
        : event.type === "LoanRepaid"
          ? "success"
          : event.type === "EscrowReleased"
            ? "success"
            : "info",
    read: false,
  };

  return {
    ...data,
    notifications: [newNotification, ...data.notifications].slice(0, 50),
  };
}

function notificationTitleFor(type: DashboardEventType): string {
  switch (type) {
    case "LoanCreated":
      return "New loan originated";
    case "LoanRepaid":
      return "Loan fully repaid";
    case "LoanDefaulted":
      return "Loan in default";
    case "EscrowReleased":
      return "Escrow released";
    case "RiskScoreUpdated":
      return "Risk score updated";
    case "NotificationCreated":
      return "New notification";
  }
}

function notificationBodyFor(event: DashboardRealtimeEvent): string {
  const payload = event.payload ?? {};
  if (typeof payload.message === "string") return payload.message;
  if (typeof payload.loanId === "string") return `Reference: loan ${payload.loanId}.`;
  return "Live update received from the network.";
}

// ── Hook ───────────────────────────────────────────────────────────────────

export function useDashboard(
  options: UseDashboardOptions = {},
): UseDashboardReturn {
  const {
    walletAddress = null,
    enableRealtime = true,
    refreshIntervalMs = 60_000,
  } = options;

  const [loans, setLoans] = useState<Loan[]>([]);
  const [notifications, setNotifications] = useState<DashboardNotification[]>(
    INITIAL_NOTIFICATIONS,
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [connected, setConnected] = useState(false);

  const txContext = useTransactionStatus();
  const eventQueue = useRef<DashboardRealtimeEvent[]>([]);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      // TODO: replace with real GET /api/v1/dashboard once the endpoint lands.
      await new Promise((resolve) => setTimeout(resolve, 350));
      setLoans(MOCK_LOANS);
      setLastUpdated(new Date());
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load dashboard");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    if (!refreshIntervalMs) return;
    const id = setInterval(() => {
      refresh();
    }, refreshIntervalMs);
    return () => clearInterval(id);
  }, [refresh, refreshIntervalMs]);

  // WebSocket connection for real-time updates. Mirrors the reconnect behaviour
  // of useWebSocket so consumers get the same backoff guarantees.
  useEffect(() => {
    if (!enableRealtime) return;

    let ws: WebSocket | null = null;
    let failures = 0;
    let stopped = false;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const connect = () => {
      if (stopped) return;
      if (typeof window === "undefined" || typeof WebSocket === "undefined")
        return;

      try {
        ws = new WebSocket("/ws");
      } catch {
        failures += 1;
        timer = setTimeout(connect, computeReconnectDelay(failures));
        return;
      }

      ws.onopen = () => {
        if (stopped) {
          ws?.close();
          return;
        }
        failures = 0;
        setConnected(true);
      };

      ws.onmessage = (ev: MessageEvent) => {
        if (stopped) return;
        try {
          const data = JSON.parse(ev.data as string) as DashboardRealtimeEvent;
          if (data && typeof data.type === "string") {
            eventQueue.current.push(data);
            setNotifications((prev) =>
              applyRealtimeEvent(
                {
                  portfolio: aggregatePortfolio(loans),
                  loans,
                  activity: [],
                  notifications: prev,
                  metrics: [],
                },
                data,
              ).notifications,
            );
          }
        } catch {
          // Ignore malformed payloads.
        }
      };

      ws.onclose = () => {
        if (stopped) return;
        setConnected(false);
        failures += 1;
        if (failures > 3) return;
        timer = setTimeout(connect, computeReconnectDelay(failures));
      };

      ws.onerror = () => {
        // onclose will fire after onerror.
      };
    };

    connect();

    return () => {
      stopped = true;
      if (timer) clearTimeout(timer);
      if (ws) {
        ws.onopen = null;
        ws.onmessage = null;
        ws.onclose = null;
        ws.onerror = null;
        ws.close();
      }
      setConnected(false);
    };
    // walletAddress is included so that switching wallets resets the socket.
  }, [enableRealtime, walletAddress, loans]);

  const portfolio = useMemo(() => aggregatePortfolio(loans), [loans]);
  const metrics = useMemo(() => buildMetricsSeries(loans), [loans]);

  const activity = useMemo<ActivityItem[]>(() => {
    const fromLoans = buildActivityFeed(loans);
    const fromTransactions: ActivityItem[] = txContext.transactions.map((tx) => ({
      id: tx.id,
      title: tx.type,
      description: tx.description,
      timestamp: new Date(tx.timestamp).toISOString(),
      category: "system",
      status:
        tx.status === "success"
          ? "success"
          : tx.status === "failed"
            ? "failed"
            : "pending",
    }));
    return [...fromTransactions, ...fromLoans]
      .sort((a, b) => (a.timestamp < b.timestamp ? 1 : -1))
      .slice(0, 25);
  }, [loans, txContext.transactions]);

  const markNotificationRead = useCallback((id: string) => {
    setNotifications((prev) =>
      prev.map((n) => (n.id === id ? { ...n, read: true } : n)),
    );
  }, []);

  const clearNotifications = useCallback(() => {
    setNotifications([]);
  }, []);

  return {
    portfolio,
    loans,
    activity,
    notifications,
    metrics,
    loading,
    error,
    connected,
    lastUpdated,
    refresh,
    markNotificationRead,
    clearNotifications,
  };
}

"use client";

import { useState, useCallback } from "react";

export interface PlatformStats {
  totalEscrows: number;
  fundedEscrows: number;
  releasedEscrows: number;
  disputedEscrows: number;
  totalLoans: number;
  activeLoans: number;
  defaultedLoans: number;
  totalVolumeUSDC: string;
  totalUsers: number;
  activeWallets: number;
  governanceProposals: number;
  participationRate: number;
}

export interface ProtocolAnalytics {
  tvl: string;
  totalVolume: string;
  avgInterestRate: string;
  defaultRate: number;
}

export interface PortfolioMetric {
  date: string;
  tvl: number;
  volume: number;
  loans: number;
  escrows: number;
}

const MOCK_STATS: PlatformStats = {
  totalEscrows: 48,
  fundedEscrows: 12,
  releasedEscrows: 31,
  disputedEscrows: 2,
  totalLoans: 35,
  activeLoans: 18,
  defaultedLoans: 2,
  totalVolumeUSDC: "2450000",
  totalUsers: 124,
  activeWallets: 89,
  governanceProposals: 7,
  participationRate: 0.62,
};

const MOCK_PROTOCOL: ProtocolAnalytics = {
  tvl: "1850000",
  totalVolume: "2450000",
  avgInterestRate: "0.0785",
  defaultRate: 0.057,
};

function generatePortfolioHistory(): PortfolioMetric[] {
  const points: PortfolioMetric[] = [];
  const now = Date.now();
  for (let i = 89; i >= 0; i--) {
    const date = new Date(now - i * 24 * 60 * 60 * 1000).toISOString().split("T")[0];
    points.push({
      date,
      tvl: 1200000 + Math.random() * 800000,
      volume: 20000 + Math.random() * 60000,
      loans: 10 + Math.floor(Math.random() * 15),
      escrows: 5 + Math.floor(Math.random() * 10),
    });
  }
  return points;
}

export function useAnalytics() {
  const [stats, setStats] = useState<PlatformStats | null>(null);
  const [protocol, setProtocol] = useState<ProtocolAnalytics | null>(null);
  const [portfolio, setPortfolio] = useState<PortfolioMetric[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStats = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/v1/analytics");
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setStats(data);
    } catch {
      setStats(MOCK_STATS);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchProtocol = useCallback(async () => {
    try {
      const res = await fetch("/api/v1/analytics/protocol");
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setProtocol(data);
    } catch {
      setProtocol(MOCK_PROTOCOL);
    }
  }, []);

  const fetchPortfolio = useCallback(async () => {
    setLoading(true);
    try {
      // Portfolio history endpoint not yet implemented server-side; use generated mock
      setPortfolio(generatePortfolioHistory());
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    stats,
    protocol,
    portfolio,
    loading,
    error,
    fetchStats,
    fetchProtocol,
    fetchPortfolio,
  };
}

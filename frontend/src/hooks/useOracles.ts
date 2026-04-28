"use client";

import { useState, useCallback } from "react";

export type OracleType = "GENERAL" | "PRICE" | "WEATHER" | "SHIPPING" | "QUALITY";

export interface OracleReputation {
  score: number;
  accuracy: number;
  reliability: number;
  responsiveness: number;
  totalVotes: number;
  positiveVotes: number;
  negativeVotes: number;
}

export interface OracleStake {
  amount: number;
  assetCode: string;
  stakedAt: string;
  rewardAmount: number;
  slashedAmount: number;
}

export interface Oracle {
  id: string;
  address: string;
  isActive: boolean;
  oracleType: OracleType;
  registeredAt: string;
  totalConfirmations: number;
  successfulConfirmations: number;
  failedConfirmations: number;
  lastActiveAt?: string;
  reputation?: OracleReputation;
  stake?: OracleStake;
}

export interface OracleNetworkStatus {
  activeOracles: number;
  totalOracles: number;
  networkHealth: number;
  recentActivity: number;
  averageReputation: { score: number; accuracy: number; reliability: number };
  oraclesByType: Record<string, number>;
}

const MOCK_ORACLES: Oracle[] = [
  {
    id: "ora-001",
    address: "GBXYZ4KITP4KVFJN4OGR6MXBGHOBRCGIXAQ7ALIEQVRSWZLUQBIHIG2K",
    isActive: true,
    oracleType: "SHIPPING",
    registeredAt: "2026-01-15T10:00:00Z",
    totalConfirmations: 142,
    successfulConfirmations: 138,
    failedConfirmations: 4,
    lastActiveAt: "2026-04-25T08:00:00Z",
    reputation: { score: 96, accuracy: 97.2, reliability: 95.8, responsiveness: 98.1, totalVotes: 142, positiveVotes: 138, negativeVotes: 4 },
    stake: { amount: 5000, assetCode: "USDC", stakedAt: "2026-01-15T10:00:00Z", rewardAmount: 320, slashedAmount: 0 },
  },
  {
    id: "ora-002",
    address: "GCABC5MJUQ8LXFGV5PGRAYHNQWFS2HP5EXLSM6MWQKVB5ZDLTRCKNJ7",
    isActive: true,
    oracleType: "QUALITY",
    registeredAt: "2026-02-01T09:00:00Z",
    totalConfirmations: 87,
    successfulConfirmations: 82,
    failedConfirmations: 5,
    lastActiveAt: "2026-04-24T14:00:00Z",
    reputation: { score: 88, accuracy: 94.3, reliability: 91.2, responsiveness: 89.5, totalVotes: 87, positiveVotes: 82, negativeVotes: 5 },
    stake: { amount: 3000, assetCode: "USDC", stakedAt: "2026-02-01T09:00:00Z", rewardAmount: 185, slashedAmount: 50 },
  },
  {
    id: "ora-003",
    address: "GDDEF6NKUP9EYGHW6QHSBIYZNE3XUFRSSZ7MWQLNB6CXDMQTRSXON8P",
    isActive: false,
    oracleType: "PRICE",
    registeredAt: "2025-11-10T12:00:00Z",
    totalConfirmations: 210,
    successfulConfirmations: 195,
    failedConfirmations: 15,
    reputation: { score: 72, accuracy: 92.9, reliability: 85.0, responsiveness: 78.3, totalVotes: 210, positiveVotes: 195, negativeVotes: 15 },
    stake: { amount: 2000, assetCode: "USDC", stakedAt: "2025-11-10T12:00:00Z", rewardAmount: 410, slashedAmount: 200 },
  },
];

export function useOracles() {
  const [oracles, setOracles] = useState<Oracle[]>([]);
  const [oracle, setOracle] = useState<Oracle | null>(null);
  const [networkStatus, setNetworkStatus] = useState<OracleNetworkStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchOracles = useCallback(async (isActive?: boolean) => {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams();
      if (isActive !== undefined) params.set("isActive", String(isActive));
      const res = await fetch(`/api/v1/oracles?${params}`);
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setOracles(data.data?.oracles ?? []);
    } catch {
      let filtered = [...MOCK_ORACLES];
      if (isActive !== undefined) filtered = filtered.filter((o) => o.isActive === isActive);
      setOracles(filtered);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchOracleById = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`/api/v1/oracles/${id}/reputation`);
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      const rep = data.data;
      setOracle({ ...rep.oracle, reputation: rep.reputation, stake: rep.stake });
    } catch {
      const found = MOCK_ORACLES.find((o) => o.id === id || o.address === id) ?? null;
      setOracle(found);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchNetworkStatus = useCallback(async () => {
    try {
      const res = await fetch("/api/v1/oracles/network-status");
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setNetworkStatus(data.data);
    } catch {
      setNetworkStatus({
        activeOracles: 2,
        totalOracles: 3,
        networkHealth: 87,
        recentActivity: 14,
        averageReputation: { score: 85, accuracy: 94.8, reliability: 90.7 },
        oraclesByType: { SHIPPING: 1, QUALITY: 1, PRICE: 1 },
      });
    }
  }, []);

  const registerOracle = useCallback(
    async (payload: { address: string; oracleType: OracleType; stakeAmount?: number; assetCode?: string; metadata?: Record<string, unknown> }) => {
      setLoading(true);
      setError(null);
      try {
        const res = await fetch("/api/v1/oracles", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        if (!res.ok) {
          const err = await res.json().catch(() => ({}));
          throw new Error(err.message ?? "Registration failed");
        }
        const data = await res.json();
        return data.data as Oracle;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const submitConfirmation = useCallback(
    async (payload: { oracleAddress: string; escrowId: string; eventType: string; signature: string; payload: Record<string, unknown>; nonce: string }) => {
      setLoading(true);
      try {
        const res = await fetch("/api/v1/oracles/confirmations", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        if (!res.ok) {
          const err = await res.json().catch(() => ({}));
          throw new Error(err.message ?? "Submission failed");
        }
        const data = await res.json();
        return data.data;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  return {
    oracles,
    oracle,
    networkStatus,
    loading,
    error,
    fetchOracles,
    fetchOracleById,
    fetchNetworkStatus,
    registerOracle,
    submitConfirmation,
  };
}

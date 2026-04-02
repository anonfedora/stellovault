import { useState, useEffect, useCallback, useReducer } from 'react';
import {
  parseRiskScoreResponse,
  parseHistoricalScores,
  type RiskScoreResponseV2,
  type HistoricalScore,
  type SimulateResponse,
} from '../utils/riskScoreParsers';

// ─── Public types ────────────────────────────────────────────────────────────

export interface RiskScoreBreakdown {
  label: string;
  componentKey: string; // machine key e.g. "on_chain_activity"
  value: number;        // 0–1000
  weight: number;       // 0–1
}

export interface RiskHistoryEntry {
  date: string;
  score: number;
}

export interface RiskScoreData {
  score: number;
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  breakdown: RiskScoreBreakdown[];
  history: RiskHistoryEntry[];
  confidence: number;
  calculatedAt: string;
}

export interface SimulationResult {
  currentScore: number;
  projectedScore: number;
  scoreDelta: number;
  scenarioDescription: string;
}

// ─── Hook options / return ────────────────────────────────────────────────────

export interface UseRiskScoreOptions {
  walletAddress: string | null;
  range?: '6m' | '1y' | 'all';
}

export interface UseRiskScoreReturn {
  data: RiskScoreData | null;
  history: RiskHistoryEntry[];
  historyLoading: boolean;
  historyError: string | null;
  loading: boolean;
  error: string | null;
  simulationResult: SimulationResult | null;
  simulationLoading: boolean;
  simulationError: string | null;
  activateSimulation: () => Promise<void>;
  deactivateSimulation: () => void;
  appendHistoryPoint: (entry: RiskHistoryEntry) => void;
}

// ─── History reducer ──────────────────────────────────────────────────────────

type HistoryAction =
  | { type: 'SET'; entries: RiskHistoryEntry[] }
  | { type: 'APPEND'; entry: RiskHistoryEntry };

function historyReducer(state: RiskHistoryEntry[], action: HistoryAction): RiskHistoryEntry[] {
  switch (action.type) {
    case 'SET':
      return action.entries;
    case 'APPEND':
      return [...state, action.entry];
    default:
      return state;
  }
}

// ─── Date helpers ─────────────────────────────────────────────────────────────

function getStartDate(range: '6m' | '1y' | 'all'): string | null {
  if (range === 'all') return null;
  const now = new Date();
  if (range === '6m') {
    now.setMonth(now.getMonth() - 6);
  } else {
    now.setFullYear(now.getFullYear() - 1);
  }
  return now.toISOString();
}

// ─── Hook ─────────────────────────────────────────────────────────────────────

export function useRiskScore(
  walletAddress: string | null,
  range: '6m' | '1y' | 'all' = '6m',
): UseRiskScoreReturn {
  const [data, setData] = useState<RiskScoreData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [history, dispatchHistory] = useReducer(historyReducer, []);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyError, setHistoryError] = useState<string | null>(null);

  const [simulationResult, setSimulationResult] = useState<SimulationResult | null>(null);
  const [simulationLoading, setSimulationLoading] = useState(false);
  const [simulationError, setSimulationError] = useState<string | null>(null);

  // ── 1.3 Fetch current risk score ──────────────────────────────────────────
  useEffect(() => {
    if (!walletAddress) {
      setData(null);
      setError(null);
      return;
    }

    let cancelled = false;

    const fetchScore = async () => {
      setLoading(true);
      setError(null);
      try {
        const res = await fetch(`/api/risk/${encodeURIComponent(walletAddress)}`);
        if (!res.ok) {
          throw new Error(`Failed to fetch risk score: ${res.status} ${res.statusText}`);
        }
        const raw: RiskScoreResponseV2 = await res.json();
        if (!cancelled) {
          setData(parseRiskScoreResponse(raw));
        }
      } catch (err: unknown) {
        if (!cancelled) {
          setError((err as Error).message ?? 'Failed to fetch risk score');
          setData(null);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    fetchScore();
    return () => { cancelled = true; };
  }, [walletAddress]);

  // ── 1.4 Fetch history ─────────────────────────────────────────────────────
  useEffect(() => {
    if (!walletAddress) {
      dispatchHistory({ type: 'SET', entries: [] });
      setHistoryError(null);
      return;
    }

    let cancelled = false;

    const fetchHistory = async () => {
      setHistoryLoading(true);
      setHistoryError(null);
      try {
        const params = new URLSearchParams();
        const startDate = getStartDate(range);
        if (startDate) params.set('start_date', startDate);
        // end_date defaults to now; omit for simplicity
        const url = `/api/risk/${encodeURIComponent(walletAddress)}/history?${params.toString()}`;
        const res = await fetch(url);
        if (!res.ok) {
          throw new Error(`Failed to fetch history: ${res.status} ${res.statusText}`);
        }
        const raw: HistoricalScore[] = await res.json();
        if (!cancelled) {
          dispatchHistory({ type: 'SET', entries: parseHistoricalScores(raw) });
        }
      } catch (err: unknown) {
        if (!cancelled) {
          setHistoryError((err as Error).message ?? 'Failed to fetch history');
          dispatchHistory({ type: 'SET', entries: [] });
        }
      } finally {
        if (!cancelled) setHistoryLoading(false);
      }
    };

    fetchHistory();
    return () => { cancelled = true; };
  }, [walletAddress, range]);

  // ── 1.5 Simulation ────────────────────────────────────────────────────────
  const activateSimulation = useCallback(async () => {
    if (!walletAddress || !data) return;
    setSimulationLoading(true);
    setSimulationError(null);
    try {
      const res = await fetch(`/api/risk/${encodeURIComponent(walletAddress)}/simulate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ loan_amount: 5000, currency: 'USDC' }),
      });
      if (!res.ok) {
        throw new Error(`Simulation request failed: ${res.status} ${res.statusText}`);
      }
      const raw: SimulateResponse = await res.json();
      const projectedScore =
        typeof raw.projected_score === 'number' ? raw.projected_score : 0;
      const scenarioDescription =
        typeof raw.scenario_description === 'string' ? raw.scenario_description : '';
      const currentScore = data.score;
      setSimulationResult({
        currentScore,
        projectedScore,
        scoreDelta: projectedScore - currentScore,
        scenarioDescription,
      });
    } catch (err: unknown) {
      setSimulationError((err as Error).message ?? 'Simulation failed');
      setSimulationResult(null);
    } finally {
      setSimulationLoading(false);
    }
  }, [walletAddress, data]);

  const deactivateSimulation = useCallback(() => {
    setSimulationResult(null);
    setSimulationError(null);
  }, []);

  // ── 1.6 WebSocket-driven append ───────────────────────────────────────────
  const appendHistoryPoint = useCallback((entry: RiskHistoryEntry) => {
    dispatchHistory({ type: 'APPEND', entry });
  }, []);

  return {
    data,
    history,
    historyLoading,
    historyError,
    loading,
    error,
    simulationResult,
    simulationLoading,
    simulationError,
    activateSimulation,
    deactivateSimulation,
    appendHistoryPoint,
  };
}

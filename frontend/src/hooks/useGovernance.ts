"use client";

import { useState, useCallback, useEffect, useRef } from "react";

export type ProposalStatus = "OPEN" | "PASSED" | "REJECTED" | "EXECUTED";
export type VoteType = "For" | "Against" | "Abstain";
export type ProposalType =
  | "Protocol Settings"
  | "Reward Parameters"
  | "Treasury Grant"
  | "Community Initiative";

export interface Proposal {
  id: string;
  title: string;
  description: string;
  status: ProposalStatus;
  creator: string;
  createdAt: number;
  expiresAt: number;
  votes: {
    for: number;
    against: number;
    abstain: number;
  };
  type: ProposalType | string;
  votingPower?: number; // total staked tokens eligible to vote
  quorumRequired?: number; // minimum participation threshold
  threshold?: number; // % of for-votes needed to pass
}

export interface GovernanceFilters {
  status: ProposalStatus | "ALL";
  type: string;
  sortBy: "newest" | "oldest" | "mostVotes";
}

// ─── Mock data ────────────────────────────────────────────────────────────────

const INITIAL_PROPOSALS: Proposal[] = [
  {
    id: "1",
    title: "Increase Liquidity Reward for USDC/XLM Pool",
    description:
      "Proposal to increase the daily reward distribution for the USDC/XLM pool from 5,000 to 7,500 XLM to attract more liquidity providers and deepen the market.",
    status: "OPEN",
    creator: "GE7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVA3BS",
    createdAt: Date.now() - 86400000 * 2,
    expiresAt: Date.now() + 86400000 * 3,
    votes: { for: 1250000, against: 450000, abstain: 120000 },
    type: "Reward Parameters",
    votingPower: 10000000,
    quorumRequired: 0.1,
    threshold: 0.667,
  },
  {
    id: "2",
    title: "Update Governance Veto Threshold",
    description:
      "Set the veto threshold for critical protocol changes to 33.3% of total staked tokens to ensure decentralization and prevent hostile takeovers.",
    status: "PASSED",
    creator: "GC3QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVP8Q1",
    createdAt: Date.now() - 86400000 * 10,
    expiresAt: Date.now() - 86400000 * 5,
    votes: { for: 5000000, against: 1000000, abstain: 500000 },
    type: "Protocol Settings",
    votingPower: 10000000,
    quorumRequired: 0.1,
    threshold: 0.667,
  },
  {
    id: "3",
    title: "Allocate 100k XLM for Community Marketing",
    description:
      "Grant to fund community-led marketing initiatives for the next quarter, including local meetups, educational content, and developer outreach programs.",
    status: "REJECTED",
    creator: "GA3QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVL2X9",
    createdAt: Date.now() - 86400000 * 15,
    expiresAt: Date.now() - 86400000 * 10,
    votes: { for: 800000, against: 2500000, abstain: 300000 },
    type: "Treasury Grant",
    votingPower: 10000000,
    quorumRequired: 0.1,
    threshold: 0.667,
  },
  {
    id: "4",
    title: "Deploy Collateral Registry v2 Contract",
    description:
      "Upgrade the on-chain collateral registry to v2, enabling fractional NFT collateral and improved liquidation mechanics.",
    status: "EXECUTED",
    creator: "GB3QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVK9Z2",
    createdAt: Date.now() - 86400000 * 30,
    expiresAt: Date.now() - 86400000 * 25,
    votes: { for: 7200000, against: 800000, abstain: 200000 },
    type: "Protocol Settings",
    votingPower: 10000000,
    quorumRequired: 0.1,
    threshold: 0.667,
  },
];

// ─── WebSocket event type ─────────────────────────────────────────────────────

interface VoteUpdateEvent {
  type: "VoteUpdated";
  proposalId: string;
  votes: { for: number; against: number; abstain: number };
}

// ─── Hook ─────────────────────────────────────────────────────────────────────

export function useGovernance() {
  const [proposals, setProposals] = useState<Proposal[]>(INITIAL_PROPOSALS);
  const [votingState, setVotingState] = useState<{
    id: string | null;
    loading: boolean;
    error: string | null;
  }>({ id: null, loading: false, error: null });
  const [userVotes, setUserVotes] = useState<Record<string, VoteType>>({});
  const [wsConnected, setWsConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const stoppedRef = useRef(false);

  // ── Real-time vote updates via WebSocket ──────────────────────────────────
  useEffect(() => {
    stoppedRef.current = false;
    let failures = 0;

    function connect() {
      if (stoppedRef.current) return;
      const ws = new WebSocket("/ws");
      wsRef.current = ws;

      ws.onopen = () => {
        if (stoppedRef.current) { ws.close(); return; }
        failures = 0;
        setWsConnected(true);
      };

      ws.onmessage = (event: MessageEvent) => {
        if (stoppedRef.current) return;
        try {
          const data = JSON.parse(event.data as string) as VoteUpdateEvent;
          if (data.type === "VoteUpdated") {
            setProposals((prev) =>
              prev.map((p) =>
                p.id === data.proposalId ? { ...p, votes: data.votes } : p,
              ),
            );
          }
        } catch {
          // ignore malformed messages
        }
      };

      ws.onclose = () => {
        if (stoppedRef.current) return;
        setWsConnected(false);
        failures += 1;
        if (failures < 3) {
          const delay = Math.min(Math.pow(2, failures - 1) * 1000, 30000);
          reconnectTimer.current = setTimeout(connect, delay);
        }
      };

      ws.onerror = () => { /* onclose handles reconnect */ };
    }

    connect();

    return () => {
      stoppedRef.current = true;
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
      if (wsRef.current) {
        wsRef.current.onopen = null;
        wsRef.current.onmessage = null;
        wsRef.current.onclose = null;
        wsRef.current.onerror = null;
        wsRef.current.close();
      }
      setWsConnected(false);
    };
  }, []);

  // ── Voting ────────────────────────────────────────────────────────────────
  const vote = useCallback(
    async (proposalId: string, type: VoteType) => {
      if (userVotes[proposalId]) {
        setVotingState({
          id: proposalId,
          loading: false,
          error: "You have already voted on this proposal.",
        });
        return;
      }

      setVotingState({ id: proposalId, loading: true, error: null });

      try {
        // Simulate XDR signing flow (replace with real Freighter signTransaction)
        await new Promise((resolve) => setTimeout(resolve, 1500));

        setProposals((prev) =>
          prev.map((p) => {
            if (p.id !== proposalId) return p;
            const newVotes = { ...p.votes };
            // Use a realistic voting weight (100k tokens per vote for mock)
            if (type === "For") newVotes.for += 100000;
            if (type === "Against") newVotes.against += 100000;
            if (type === "Abstain") newVotes.abstain += 100000;
            return { ...p, votes: newVotes };
          }),
        );

        setUserVotes((prev) => ({ ...prev, [proposalId]: type }));
        setVotingState({ id: null, loading: false, error: null });
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : "Failed to cast vote";
        setVotingState({ id: proposalId, loading: false, error: message });
      }
    },
    [userVotes],
  );

  // ── Create proposal ───────────────────────────────────────────────────────
  const createProposal = useCallback(
    async (data: {
      title: string;
      description: string;
      type: string;
      duration: number;
    }) => {
      const newProposal: Proposal = {
        id: Math.random().toString(36).substring(2, 9),
        title: data.title,
        description: data.description,
        status: "OPEN",
        creator: "ME...USER",
        createdAt: Date.now(),
        expiresAt: Date.now() + 86400000 * data.duration,
        votes: { for: 0, against: 0, abstain: 0 },
        type: data.type,
        votingPower: 10000000,
        quorumRequired: 0.1,
        threshold: 0.667,
      };
      setProposals((prev) => [newProposal, ...prev]);
      return newProposal;
    },
    [],
  );

  return {
    proposals,
    votingState,
    userVotes,
    wsConnected,
    vote,
    createProposal,
  };
}

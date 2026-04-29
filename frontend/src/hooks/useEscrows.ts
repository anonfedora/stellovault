"use client";

import { useEffect, useMemo, useState } from "react";

export type EscrowStatus =
  | "draft"
  | "funded"
  | "in_transit"
  | "inspection"
  | "released"
  | "disputed";

export type EscrowEvent = {
  id: string;
  title: string;
  description: string;
  timestamp: string;
  status: EscrowStatus;
};

export type Escrow = {
  id: string;
  title: string;
  buyer: string;
  seller: string;
  amount: number;
  asset: string;
  status: EscrowStatus;
  updatedAt: string;
  dueDate: string;
  deliveryRoute: string;
  oracle: string;
  events: EscrowEvent[];
};

export type EscrowFilters = {
  status: "all" | EscrowStatus;
  party: string;
  dateRange: "all" | "7d" | "30d";
};

const MOCK_ESCROWS: Escrow[] = [
  {
    id: "ESC-2048",
    title: "Cocoa export invoice",
    buyer: "Accra Commodity Buyers",
    seller: "Lagos Agro Cooperative",
    amount: 48200,
    asset: "USDC",
    status: "inspection",
    updatedAt: "2026-04-24T14:30:00Z",
    dueDate: "2026-05-04",
    deliveryRoute: "Tema Port -> Apapa Port",
    oracle: "Shipment Oracle A",
    events: [
      {
        id: "e1",
        title: "Escrow created",
        description: "Trade terms and release conditions were accepted.",
        timestamp: "2026-04-20T09:00:00Z",
        status: "draft",
      },
      {
        id: "e2",
        title: "Buyer funded escrow",
        description: "48,200 USDC locked in the Soroban escrow contract.",
        timestamp: "2026-04-21T11:15:00Z",
        status: "funded",
      },
      {
        id: "e3",
        title: "Shipment verified",
        description: "Carrier scan confirmed cargo in transit.",
        timestamp: "2026-04-23T16:40:00Z",
        status: "in_transit",
      },
      {
        id: "e4",
        title: "Inspection pending",
        description: "Buyer has 48 hours to approve delivery documents.",
        timestamp: "2026-04-24T14:30:00Z",
        status: "inspection",
      },
    ],
  },
  {
    id: "ESC-2051",
    title: "Solar equipment receivable",
    buyer: "Nairobi Renewables Ltd",
    seller: "CapeTech Supply",
    amount: 73000,
    asset: "USDC",
    status: "funded",
    updatedAt: "2026-04-25T08:20:00Z",
    dueDate: "2026-05-12",
    deliveryRoute: "Cape Town -> Nairobi",
    oracle: "IoT Seal Oracle",
    events: [
      {
        id: "e1",
        title: "Escrow created",
        description: "Counterparties signed the milestone schedule.",
        timestamp: "2026-04-24T12:00:00Z",
        status: "draft",
      },
      {
        id: "e2",
        title: "Funds locked",
        description: "Buyer funded escrow awaiting dispatch confirmation.",
        timestamp: "2026-04-25T08:20:00Z",
        status: "funded",
      },
    ],
  },
  {
    id: "ESC-2037",
    title: "Textile shipment settlement",
    buyer: "Dakar Retail Group",
    seller: "Kano Textile Works",
    amount: 31500,
    asset: "XLM",
    status: "released",
    updatedAt: "2026-04-18T17:00:00Z",
    dueDate: "2026-04-19",
    deliveryRoute: "Kano -> Dakar",
    oracle: "Customs Oracle B",
    events: [
      {
        id: "e1",
        title: "Escrow funded",
        description: "Buyer funds were locked pending delivery.",
        timestamp: "2026-04-12T10:30:00Z",
        status: "funded",
      },
      {
        id: "e2",
        title: "Released",
        description: "Delivery approved and funds released to seller.",
        timestamp: "2026-04-18T17:00:00Z",
        status: "released",
      },
    ],
  },
  {
    id: "ESC-2032",
    title: "Copper cathode tranche",
    buyer: "Casablanca Metals",
    seller: "Lusaka Mining Export",
    amount: 126400,
    asset: "USDC",
    status: "disputed",
    updatedAt: "2026-04-19T13:10:00Z",
    dueDate: "2026-04-27",
    deliveryRoute: "Lusaka -> Casablanca",
    oracle: "Warehouse Oracle C",
    events: [
      {
        id: "e1",
        title: "Escrow funded",
        description: "Funds locked with dispute arbitration enabled.",
        timestamp: "2026-04-15T09:45:00Z",
        status: "funded",
      },
      {
        id: "e2",
        title: "Dispute opened",
        description: "Buyer challenged warehouse weight certificate.",
        timestamp: "2026-04-19T13:10:00Z",
        status: "disputed",
      },
    ],
  },
];

export function useEscrows(filters?: EscrowFilters) {
  const [escrows, setEscrows] = useState(MOCK_ESCROWS);
  const [lastSyncAt, setLastSyncAt] = useState(new Date().toISOString());
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const interval = window.setInterval(() => {
      setLastSyncAt(new Date().toISOString());
      setNow(Date.now());
    }, 15000);

    return () => window.clearInterval(interval);
  }, []);

  const filteredEscrows = useMemo(() => {
    return escrows.filter((escrow) => {
      if (filters?.status && filters.status !== "all" && escrow.status !== filters.status) {
        return false;
      }

      if (filters?.party) {
        const value = filters.party.toLowerCase();
        const partyMatch =
          escrow.buyer.toLowerCase().includes(value) ||
          escrow.seller.toLowerCase().includes(value) ||
          escrow.id.toLowerCase().includes(value);

        if (!partyMatch) return false;
      }

      if (filters?.dateRange && filters.dateRange !== "all") {
        const days = filters.dateRange === "7d" ? 7 : 30;
        const updatedAt = new Date(escrow.updatedAt).getTime();
        const cutoff = now - days * 24 * 60 * 60 * 1000;
        if (updatedAt < cutoff) return false;
      }

      return true;
    });
  }, [escrows, filters, now]);

  const getEscrowById = (id: string) => escrows.find((escrow) => escrow.id === id);

  return {
    escrows: filteredEscrows,
    allEscrows: escrows,
    setEscrows,
    getEscrowById,
    lastSyncAt,
  };
}

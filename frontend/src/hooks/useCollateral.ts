"use client";

import { useState, useCallback } from "react";

export type CollateralStatus = "LOCKED" | "RELEASED" | "LIQUIDATED";
export type CollateralAssetType = "INVOICE" | "COMMODITY" | "RECEIVABLE" | "INVENTORY" | "REAL_ESTATE";

export interface CollateralDocument {
  name: string;
  hash: string;
  mimeType?: string;
  size?: number;
}

export interface Collateral {
  id: string;
  escrowId: string;
  assetCode: string;
  assetType?: CollateralAssetType;
  amount: string;
  metadataHash?: string;
  status: CollateralStatus;
  description?: string;
  issuer?: string;
  documents?: CollateralDocument[];
  stellarAssetCode?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CollateralType {
  type: CollateralAssetType;
  maxLtvRatio: number;
  description: string;
}

export interface LTVData {
  collateralId: string;
  collateralValue: number;
  totalLoanAmount: number;
  currentLtv: number;
  maxLtvRatio: number;
  maxLoanAmount: number;
  availableCredit: number;
}

const MOCK_COLLATERALS: Collateral[] = [
  {
    id: "col-001",
    escrowId: "esc-001",
    assetCode: "INVOICE",
    assetType: "INVOICE",
    amount: "50000",
    metadataHash: "QmXyz123",
    status: "LOCKED",
    description: "Export invoice for cocoa shipment",
    issuer: "Lagos Agro Cooperative",
    stellarAssetCode: "INVOC001",
    documents: [{ name: "invoice.pdf", hash: "QmDoc1", mimeType: "application/pdf" }],
    createdAt: "2026-04-20T09:00:00Z",
    updatedAt: "2026-04-20T09:00:00Z",
  },
  {
    id: "col-002",
    escrowId: "esc-002",
    assetCode: "COMMODITY",
    assetType: "COMMODITY",
    amount: "75000",
    metadataHash: "QmAbc456",
    status: "LOCKED",
    description: "Solar equipment inventory",
    issuer: "CapeTech Supply",
    stellarAssetCode: "COMMB002",
    documents: [
      { name: "warehouse_receipt.pdf", hash: "QmDoc2", mimeType: "application/pdf" },
      { name: "quality_cert.jpg", hash: "QmDoc3", mimeType: "image/jpeg" },
    ],
    createdAt: "2026-04-22T10:00:00Z",
    updatedAt: "2026-04-22T10:00:00Z",
  },
  {
    id: "col-003",
    escrowId: "esc-003",
    assetCode: "RECEIVABLE",
    assetType: "RECEIVABLE",
    amount: "30000",
    metadataHash: "QmDef789",
    status: "RELEASED",
    description: "Textile receivable",
    issuer: "Kano Textile Works",
    stellarAssetCode: "RECEC003",
    documents: [],
    createdAt: "2026-04-10T08:00:00Z",
    updatedAt: "2026-04-18T17:00:00Z",
  },
];

export function useCollateral() {
  const [collaterals, setCollaterals] = useState<Collateral[]>([]);
  const [collateral, setCollateral] = useState<Collateral | null>(null);
  const [collateralTypes, setCollateralTypes] = useState<CollateralType[]>([]);
  const [ltv, setLtv] = useState<LTVData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchCollaterals = useCallback(async (escrowId?: string, status?: string) => {
    setLoading(true);
    setError(null);
    try {
      const params = new URLSearchParams();
      if (escrowId) params.set("escrowId", escrowId);
      if (status) params.set("status", status);
      const res = await fetch(`/api/v1/collateral?${params}`);
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setCollaterals(data.data?.items ?? []);
    } catch {
      let filtered = [...MOCK_COLLATERALS];
      if (status) filtered = filtered.filter((c) => c.status === status);
      setCollaterals(filtered);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchCollateralById = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`/api/v1/collateral/${id}/metadata`);
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setCollateral(data.data);
    } catch {
      const found = MOCK_COLLATERALS.find((c) => c.id === id) ?? null;
      setCollateral(found);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchCollateralTypes = useCallback(async () => {
    try {
      const res = await fetch("/api/v1/collateral/types");
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setCollateralTypes(data.data ?? []);
    } catch {
      setCollateralTypes([
        { type: "INVOICE", maxLtvRatio: 80, description: "Trade invoices and receivables" },
        { type: "COMMODITY", maxLtvRatio: 70, description: "Physical commodities" },
        { type: "RECEIVABLE", maxLtvRatio: 75, description: "Accounts receivable" },
        { type: "INVENTORY", maxLtvRatio: 60, description: "Warehouse inventory" },
        { type: "REAL_ESTATE", maxLtvRatio: 65, description: "Real estate assets" },
      ]);
    }
  }, []);

  const fetchLTV = useCallback(async (id: string) => {
    try {
      const res = await fetch(`/api/v1/collateral/${id}/ltv`);
      if (!res.ok) throw new Error("API unavailable");
      const data = await res.json();
      setLtv(data.data);
    } catch {
      setLtv({
        collateralId: id,
        collateralValue: 50000,
        totalLoanAmount: 25000,
        currentLtv: 50,
        maxLtvRatio: 80,
        maxLoanAmount: 40000,
        availableCredit: 15000,
      });
    }
  }, []);

  const tokenizeAsset = useCallback(
    async (payload: {
      userId: string;
      escrowId: string;
      assetData: { assetType: string; assetCode?: string; amount: number; description?: string; issuer?: string };
      documents?: CollateralDocument[];
    }) => {
      setLoading(true);
      setError(null);
      try {
        const res = await fetch("/api/v1/collateral", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        if (!res.ok) {
          const err = await res.json().catch(() => ({}));
          throw new Error(err.message ?? "Tokenization failed");
        }
        const data = await res.json();
        return data.data as Collateral;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const updateValuation = useCallback(async (id: string, newUSDValue: number) => {
    setLoading(true);
    try {
      const res = await fetch(`/api/v1/collateral/${id}/valuation`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ newUSDValue }),
      });
      if (!res.ok) throw new Error("Update failed");
      const data = await res.json();
      return data.data as Collateral;
    } finally {
      setLoading(false);
    }
  }, []);

  const verifyCollateral = useCallback(
    async (id: string, verificationData: { verifier: string; method: string; result: string }) => {
      setLoading(true);
      try {
        const res = await fetch(`/api/v1/collateral/${id}/verify`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(verificationData),
        });
        if (!res.ok) throw new Error("Verification failed");
        const data = await res.json();
        return data.data;
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  return {
    collaterals,
    collateral,
    collateralTypes,
    ltv,
    loading,
    error,
    fetchCollaterals,
    fetchCollateralById,
    fetchCollateralTypes,
    fetchLTV,
    tokenizeAsset,
    updateValuation,
    verifyCollateral,
  };
}

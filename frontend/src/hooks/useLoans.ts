"use client";

import { useState, useCallback } from "react";
import type { Loan, LoanStatus } from "@/types";

// ── Mock data for development ──────────────────────────────────────────
const MOCK_REPAYMENTS = [
  {
    id: "rep-1",
    loanId: "loan-1",
    amount: 5200,
    paidAt: new Date("2025-12-15"),
    txHash: "abc123def456",
  },
  {
    id: "rep-2",
    loanId: "loan-1",
    amount: 5200,
    paidAt: new Date("2026-01-15"),
    txHash: "def456ghi789",
  },
  {
    id: "rep-3",
    loanId: "loan-2",
    amount: 3100,
    paidAt: new Date("2026-01-10"),
    txHash: "ghi789jkl012",
  },
];

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
    status: "ACTIVE",
    repayments: [MOCK_REPAYMENTS[0], MOCK_REPAYMENTS[1]],
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
    status: "ACTIVE",
    repayments: [MOCK_REPAYMENTS[2]],
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
    status: "PENDING",
    repayments: [],
    createdAt: new Date("2026-02-01"),
    maturityDate: new Date("2026-05-01"),
  },
  {
    id: "loan-4",
    borrower: "GBXYZ4KITP4KVFJN4OGR6MXBGHOBRCGIXAQ7ALIEQVRSWZLUQBIHIG2K",
    collateralTokenId: "ct-004",
    collateralAssetType: "INVOICE",
    collateralValue: 100000,
    principal: 60000,
    interestRate: 6.5,
    termMonths: 18,
    status: "REPAID",
    repayments: [
      {
        id: "rep-r1",
        loanId: "loan-4",
        amount: 63900,
        paidAt: new Date("2025-10-01"),
        txHash: "mno345pqr678",
      },
    ],
    createdAt: new Date("2024-04-01"),
    maturityDate: new Date("2025-10-01"),
  },
  {
    id: "loan-5",
    borrower: "GCABC5MJUQ8LXFGV5PGRAYHNQWFS2HP5EXLSM6MWQKVB5ZDLTRCKNJ7",
    collateralTokenId: "ct-005",
    collateralAssetType: "COMMODITY",
    collateralValue: 20000,
    principal: 12000,
    interestRate: 10.0,
    termMonths: 6,
    status: "DEFAULTED",
    repayments: [
      {
        id: "rep-d1",
        loanId: "loan-5",
        amount: 2100,
        paidAt: new Date("2025-06-15"),
        txHash: "stu901vwx234",
      },
    ],
    createdAt: new Date("2025-01-01"),
    maturityDate: new Date("2025-07-01"),
  },
];

// ── Hook ────────────────────────────────────────────────────────────────
interface UseLoansReturn {
  loans: Loan[];
  loan: Loan | null;
  loading: boolean;
  error: string | null;
  totalPages: number;
  fetchLoans: (
    status?: LoanStatus | "ALL",
    page?: number,
    limit?: number,
  ) => Promise<void>;
  fetchLoanById: (id: string) => Promise<void>;
}

export const useLoans = (): UseLoansReturn => {
  const [loans, setLoans] = useState<Loan[]>([]);
  const [loan, setLoan] = useState<Loan | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [totalPages, setTotalPages] = useState(1);

  const fetchLoans = useCallback(
    async (status?: LoanStatus | "ALL", page = 1, limit = 10) => {
      setLoading(true);
      setError(null);
      try {
        // TODO: Replace with real API call to GET /api/v1/loans
        // const params = new URLSearchParams({ page: String(page), limit: String(limit) });
        // if (status && status !== 'ALL') params.set('status', status);
        // const res = await fetch(`/api/v1/loans?${params}`);
        // const data = await res.json();

        await new Promise((r) => setTimeout(r, 400)); // simulate network

        let filtered = [...MOCK_LOANS];
        if (status && status !== "ALL") {
          filtered = filtered.filter((l) => l.status === status);
        }
        const start = (page - 1) * limit;
        const paginated = filtered.slice(start, start + limit);

        setLoans(paginated);
        setTotalPages(Math.max(1, Math.ceil(filtered.length / limit)));
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch loans");
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const fetchLoanById = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      // TODO: Replace with real API call to GET /api/v1/loans/:id
      await new Promise((r) => setTimeout(r, 300));
      const found = MOCK_LOANS.find((l) => l.id === id) ?? null;
      if (!found) throw new Error("Loan not found");
      setLoan(found);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch loan");
    } finally {
      setLoading(false);
    }
  }, []);

  return { loans, loan, loading, error, totalPages, fetchLoans, fetchLoanById };
};

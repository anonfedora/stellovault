// TypeScript type definitions for StelloVault frontend

export interface User {
  id: string;
  address: string;
  name?: string;
  email?: string;
  role: "buyer" | "seller" | "oracle" | "admin";
}

export interface CollateralToken {
  id: string;
  owner: string;
  assetType: "INVOICE" | "COMMODITY" | "RECEIVABLE";
  assetValue: number;
  metadata: string;
  fractionalShares: number;
  createdAt: Date;
  status: "active" | "locked" | "burned";
}

export interface TradeEscrow {
  id: string;
  buyer: User;
  seller: User;
  collateralTokenId: string;
  amount: number;
  status: "pending" | "active" | "released" | "cancelled";
  oracleAddress: string;
  releaseConditions: string;
  createdAt: Date;
  releasedAt?: Date;
}

export interface Transaction {
  id: string;
  type: "tokenize" | "escrow_create" | "escrow_release" | "transfer";
  from: string;
  to: string;
  amount: number;
  timestamp: Date;
  status: "pending" | "confirmed" | "failed";
  txHash?: string;
}

export interface DashboardStats {
  totalCollateralValue: number;
  activeEscrows: number;
  completedTrades: number;
  totalUsers: number;
}

export type LoanStatus = "PENDING" | "ACTIVE" | "REPAID" | "DEFAULTED";

export interface Repayment {
  id: string;
  loanId: string;
  amount: number;
  paidAt: Date;
  txHash: string;
}

export interface Loan {
  id: string;
  borrower: string;
  collateralTokenId: string;
  collateralAssetType: "INVOICE" | "COMMODITY" | "RECEIVABLE";
  collateralValue: number;
  principal: number;
  interestRate: number;
  termMonths: number;
  status: LoanStatus;
  repayments: Repayment[];
  createdAt: Date;
  maturityDate: Date;
  xdr?: string;
}

export interface LoanFormData {
  selectedCollateralId: string;
  principal: number;
  termMonths: number;
  interestRate: number;
}

// TypeScript type definitions for StelloVault frontend
import type { TransactionStatus } from './loan';
// Re-export centralized types and runtime enums for consistency
export type {
  Repayment,
  Loan,
  LoanFormData,
  PaymentTransaction,
} from './loan';

export { 
  PaymentStatus,
  LoanStatus,
  TransactionStatus,
} from "./loan";

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
  status: TransactionStatus;
  txHash?: string;
}

export interface DashboardStats {
  totalCollateralValue: number;
  activeEscrows: number;
  completedTrades: number;
  totalUsers: number;
}

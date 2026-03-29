/**
 * Frontend types for loan and payment-related functionality
 * Defines centralized enums for payment statuses
 */

/**
 * Payment Status Enum
 * 
 * Lifecycle: PENDING -> PROCESSING -> COMPLETED | FAILED
 * 
 * Status Flow:
 * PENDING: Initial state when payment is initiated but not yet processed
 * PROCESSING: Payment is being processed by the system
 * COMPLETED: Payment successfully processed and finalized
 * FAILED: Payment processing failed and requires retry or manual intervention
 */
export enum PaymentStatus {
  PENDING = 'PENDING',
  PROCESSING = 'PROCESSING', 
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED'
}

/**
 * Loan Status Enum
 * 
 * Lifecycle: PENDING -> ACTIVE -> REPAID | DEFAULTED
 * 
 * Status Flow:
 * PENDING: Loan requested but not yet funded
 * ACTIVE: Loan funded and borrower has access to funds
 * REPAID: Loan fully repaid with interest
 * DEFAULTED: Loan failed to be repaid by due date
 */
export enum LoanStatus {
  PENDING = 'PENDING',
  ACTIVE = 'ACTIVE',
  REPAID = 'REPAID',
  DEFAULTED = 'DEFAULTED'
}

/**
 * Transaction Status Enum
 * 
 * Lifecycle: PENDING -> PROCESSING -> COMPLETED | FAILED
 * 
 * Status Flow:
 * PENDING: Transaction queued but not yet processed
 * PROCESSING: Transaction being processed by the network
 * COMPLETED: Transaction successfully confirmed on blockchain
 * FAILED: Transaction failed and requires retry
 */
export enum TransactionStatus {
  PENDING = 'PENDING',
  PROCESSING = 'PROCESSING',
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED'
}

/**
 * Type aliases for backward compatibility and clearer naming
 */
export type PaymentStatusType = PaymentStatus;
export type LoanStatusType = LoanStatus;
export type TransactionStatusType = TransactionStatus;

/**
 * Status transition validation rules
 */
export const StatusTransitions = {
  [PaymentStatus.PENDING]: [PaymentStatus.PROCESSING, PaymentStatus.FAILED],
  [PaymentStatus.PROCESSING]: [PaymentStatus.COMPLETED, PaymentStatus.FAILED],
  [PaymentStatus.COMPLETED]: [], // Terminal state
  [PaymentStatus.FAILED]: [PaymentStatus.PENDING], // Can retry from failed
} as const;

/**
 * Helper function to validate status transitions
 */
export function isValidStatusTransition(
  currentStatus: PaymentStatus,
  newStatus: PaymentStatus
): boolean {
  const allowedTransitions: PaymentStatus[] = [...StatusTransitions[currentStatus]];
  return allowedTransitions.includes(newStatus);
}

/**
 * Helper function to check if status is terminal (no further transitions possible)
 */
export function isTerminalStatus(status: PaymentStatus): boolean {
  return StatusTransitions[status].length === 0;
}

// Additional frontend-specific interfaces that use the centralized enums

export interface Repayment {
  id: string;
  loanId: string;
  amount: number;
  paidAt: Date;
  txHash: string;
  status: TransactionStatus;
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

export interface PaymentTransaction {
  id: string;
  loanId: string;
  amount: number;
  status: PaymentStatus;
  createdAt: Date;
  completedAt?: Date;
  txHash?: string;
  errorMessage?: string;
}

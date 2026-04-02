/**
 * Centralized payment status definitions for StelloVault
 * 
 * This file serves as the single source of truth for all payment-related
 * statuses across the entire codebase (frontend, backend, and database).
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

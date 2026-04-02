import { randomUUID } from "crypto";
import {
  ForbiddenError,
  NotFoundError,
  ValidationError,
} from "../config/errors";
import { contracts } from "../config/contracts";
import { xdr } from "@stellar/stellar-sdk";
import contractService from "./contract.service";
import { prisma } from "./database.service";
import websocketService from "./websocket.service";
import { env } from "../config/env";
import Decimal from "decimal.js";
import eventMonitoringService from "./event-monitoring.service";

const MIN_COLLATERAL_RATIO = new Decimal("1.5");
const VALID_LOAN_STATUSES = new Set([
  "PENDING",
  "ACTIVE",
  "REPAID",
  "DEFAULTED",
]);
const ZERO = new Decimal("0");

type LoanStatus = "PENDING" | "ACTIVE" | "REPAID" | "DEFAULTED";

interface IssueLoanRequest {
  requestingUserId?: string;
  borrowerId?: string;
  lenderId?: string;
  amount?: number | string;
  assetCode?: string;
  collateralAmt?: number | string;
  escrowAddress?: string;
}

interface RecordRepaymentRequest {
  requestingUserId?: string;
  loanId?: string;
  amount?: number | string;
  paidAt?: string | Date;
  sessionId?: string;
}

interface CreatePaymentRequest {
  requestingUserId?: string;
  loanId?: string;
  success_url?: string;
  successUrl?: string;
  cancel_url?: string;
  cancelUrl?: string;
  webhook_url?: string;
  webhookUrl?: string;
  webhook_secret?: string;
  webhookSecret?: string;
}

function parsePositiveDecimal(
  value: number | string | undefined,
  fieldName: string,
): Decimal {
  let parsed: Decimal;
  try {
    parsed = new Decimal(value as string | number);
  } catch {
    throw new ValidationError(`${fieldName} must be a positive number`);
  }
  if (!parsed.isFinite() || parsed.lte(ZERO)) {
    throw new ValidationError(`${fieldName} must be a positive number`);
  }
  return parsed;
}

function parseUrl(value: string | undefined, fieldName: string): string {
  if (!value?.trim()) {
    throw new ValidationError(`${fieldName} is required`);
  }

  try {
    return new URL(value).toString();
  } catch {
    throw new ValidationError(`${fieldName} must be a valid URL`);
  }
}

export class LoanService {
  async issueLoan(payload: IssueLoanRequest) {
    const requestingUserId = payload.requestingUserId?.trim();
    if (!requestingUserId) {
      throw new ValidationError("requestingUserId is required");
    }

    const borrowerId = payload.borrowerId?.trim();
    const lenderId = payload.lenderId?.trim();
    if (!borrowerId) {
      throw new ValidationError("borrowerId is required");
    }
    if (!lenderId) {
      throw new ValidationError("lenderId is required");
    }
    if (borrowerId === lenderId) {
      throw new ValidationError("Borrower and lender must be different");
    }
    if (requestingUserId !== borrowerId && requestingUserId !== lenderId) {
      throw new ForbiddenError(
        "Only the borrower or lender can create this loan",
      );
    }

    const amount = parsePositiveDecimal(payload.amount, "amount");
    const collateralAmt = parsePositiveDecimal(
      payload.collateralAmt,
      "collateralAmt",
    );
    const collateralRatio = collateralAmt.div(amount);
    if (collateralRatio.lt(MIN_COLLATERAL_RATIO)) {
      throw new ValidationError(
        `Collateral ratio must be at least ${MIN_COLLATERAL_RATIO.toFixed(2)}`,
      );
    }

    const db = prisma;
    const users = await db.user.findMany({
      where: { id: { in: [borrowerId, lenderId] } },
      select: { id: true },
    });
    if (users.length !== 2) {
      throw new ValidationError("borrowerId or lenderId does not exist");
    }
    const loanContractId = contracts.loan?.trim();
    if (!loanContractId) {
      throw new ValidationError("LOAN_CONTRACT_ID not configured");
    }

    const xdrResult = await contractService.buildContractInvokeXDR(
      loanContractId,
      "issue_loan",
      [
        borrowerId,
        lenderId,
        amount.toString(),
        collateralAmt.toString(),
        payload.assetCode || "USDC",
        payload.escrowAddress || "",
      ].map((v) => xdr.ScVal.scvString(v)),
      env.feePayer.publicKey,
    );

    const loan = await db.loan.create({
      data: {
        borrowerId,
        lenderId,
        amount: amount.toString(),
        assetCode: payload.assetCode || "USDC",
        status: "PENDING",
        interestRate: "0",
      },
    });

    websocketService.broadcastLoanUpdated(loan.id, loan.status);

    return {
      loanId: loan.id,
      xdr: xdrResult,
      loan,
    };
  }

  async getLoan(id: string) {
    const db = prisma;
    const loan = await db.loan.findUnique({
      where: { id },
      include: {
        borrower: true,
        lender: true,
        repayments: { orderBy: { createdAt: "asc" } },
      },
    });

    if (!loan) {
      throw new NotFoundError("Loan not found");
    }

    return loan;
  }

  async listLoans(borrowerId?: string, lenderId?: string, status?: string) {
    const db = prisma;
    const normalizedStatus = status?.trim().toUpperCase();
    if (normalizedStatus && !VALID_LOAN_STATUSES.has(normalizedStatus)) {
      throw new ValidationError(
        "Invalid status. Use PENDING, ACTIVE, REPAID, or DEFAULTED",
      );
    }

    const where: Record<string, string> = {};
    if (borrowerId?.trim()) where.borrowerId = borrowerId.trim();
    if (lenderId?.trim()) where.lenderId = lenderId.trim();
    if (normalizedStatus) where.status = normalizedStatus;

    return db.loan.findMany({
      where,
      include: {
        borrower: true,
        lender: true,
        repayments: true,
      },
      orderBy: { createdAt: "desc" },
    });
  }

  async createPayment(payload: CreatePaymentRequest) {
    const requestingUserId = payload.requestingUserId?.trim();
    const loanId = payload.loanId?.trim();
    if (!requestingUserId) {
      throw new ValidationError("requestingUserId is required");
    }
    if (!loanId) {
      throw new ValidationError("loanId is required");
    }

    const successUrl = parseUrl(
      payload.success_url ?? payload.successUrl,
      "success_url",
    );
    const cancelUrl = parseUrl(
      payload.cancel_url ?? payload.cancelUrl,
      "cancel_url",
    );
    const webhookUrlValue = payload.webhook_url ?? payload.webhookUrl;
    const webhookUrl = webhookUrlValue?.trim()
      ? parseUrl(webhookUrlValue, "webhook_url")
      : null;
    const webhookSecret =
      payload.webhook_secret ??
      payload.webhookSecret ??
      env.webhookSignatureSecret ??
      env.webhookSecret ??
      null;

    const db = prisma;
    const loan = await db.loan.findUnique({
      where: { id: loanId },
      include: {
        borrower: true,
        lender: true,
        repayments: true,
      },
    });
    if (!loan) {
      throw new NotFoundError("Loan not found");
    }
    if (
      requestingUserId !== loan.borrowerId &&
      requestingUserId !== loan.lenderId
    ) {
      throw new ForbiddenError(
        "Only the borrower or lender can create payment sessions",
      );
    }

    const totalRepaid = loan.repayments.reduce(
      (sum: Decimal, repayment) =>
        sum.plus(new Decimal(repayment.amount.toString())),
      ZERO,
    );
    const outstandingAmount = new Decimal(loan.amount.toString()).minus(
      totalRepaid,
    );
    if (outstandingAmount.lte(ZERO)) {
      throw new ValidationError("Loan is already fully repaid");
    }

    const sessionToken = randomUUID();
    const checkoutUrl = `${env.appBaseUrl.replace(/\/+$/, "")}/checkout/${sessionToken}`;
    const paymentSession = await db.paymentSession.create({
      data: {
        loanId,
        sessionToken,
        checkoutUrl,
        successUrl,
        cancelUrl,
        webhookUrl,
        webhookSecret,
      },
    });

    return {
      paymentId: paymentSession.id,
      sessionId: paymentSession.sessionToken,
      checkoutUrl: paymentSession.checkoutUrl,
      successUrl: paymentSession.successUrl,
      cancelUrl: paymentSession.cancelUrl,
      outstandingAmount: outstandingAmount.toString(),
    };
  }

  async recordRepayment(payload: RecordRepaymentRequest) {
    const requestingUserId = payload.requestingUserId?.trim();
    if (!requestingUserId) {
      throw new ValidationError("requestingUserId is required");
    }

    const loanId = payload.loanId?.trim();
    if (!loanId) {
      throw new ValidationError("loanId is required");
    }

    const amount = parsePositiveDecimal(payload.amount, "amount");
    let paidAt: Date | undefined;
    if (payload.paidAt) {
      paidAt = new Date(payload.paidAt);
      if (Number.isNaN(paidAt.getTime())) {
        throw new ValidationError("paidAt must be a valid date");
      }
    }

    const db = prisma;
    const result = await db.$transaction(
      async (tx: any) => {
        const loan = await tx.loan.findUnique({
          where: { id: loanId },
          include: { repayments: true, borrower: true, lender: true },
        });
        if (!loan) {
          throw new NotFoundError("Loan not found");
        }
        if (
          requestingUserId !== loan.borrowerId &&
          requestingUserId !== loan.lenderId
        ) {
          throw new ForbiddenError(
            "Only the borrower or lender can record repayments",
          );
        }
        if (loan.status === "DEFAULTED") {
          throw new ValidationError(
            "Cannot record repayment for a defaulted loan",
          );
        }

        const totalRepaid = loan.repayments.reduce(
          (sum: Decimal, repayment: { amount: string | number }) =>
            sum.plus(new Decimal(repayment.amount.toString())),
          ZERO,
        );
        const outstandingBefore = new Decimal(loan.amount.toString()).minus(
          totalRepaid,
        );
        if (outstandingBefore.lte(ZERO)) {
          throw new ValidationError("Loan is already fully repaid");
        }
        if (amount.gt(outstandingBefore)) {
          throw new ValidationError("Repayment exceeds outstanding balance");
        }

        const repayment = await tx.repayment.create({
          data: {
            loanId,
            amount: amount.toString(),
            ...(paidAt ? { paidAt } : {}),
          },
        });

        const selectedPaymentSession = payload.sessionId?.trim()
          ? await tx.paymentSession.findFirst({
              where: {
                loanId,
                sessionToken: payload.sessionId.trim(),
                status: "PENDING",
              },
              orderBy: { createdAt: "desc" },
            })
          : await tx.paymentSession.findFirst({
              where: {
                loanId,
                status: "PENDING",
              },
              orderBy: { createdAt: "desc" },
            });

        if (selectedPaymentSession) {
          await tx.paymentSession.update({
            where: { id: selectedPaymentSession.id },
            data: {
              repaymentId: repayment.id,
              status: "COMPLETED",
              completedAt: paidAt ?? new Date(),
            },
          });
        }

        const outstandingAfter = outstandingBefore.minus(amount);
        let nextStatus: LoanStatus = loan.status;
        if (outstandingAfter.eq(ZERO)) {
          nextStatus = "REPAID";
        } else if (loan.status === "PENDING") {
          nextStatus = "ACTIVE";
        }

        if (nextStatus !== loan.status) {
          await tx.loan.update({
            where: { id: loanId },
            data: { status: nextStatus },
          });

          websocketService.broadcastLoanUpdated(loanId, nextStatus);
        }

        const updatedLoan = await tx.loan.findUnique({
          where: { id: loanId },
          include: { repayments: true, borrower: true, lender: true },
        });
        if (!updatedLoan) {
          throw new NotFoundError("Loan not found");
        }

        return {
          repayment,
          paymentSession: selectedPaymentSession,
          outstandingBefore: outstandingBefore.toString(),
          outstandingAfter: outstandingAfter.toString(),
          fullyRepaid: outstandingAfter.eq(ZERO),
          loan: updatedLoan,
        };
      },
      { isolationLevel: "Serializable" },
    );

    void eventMonitoringService
      .processEvent({
        type: "PaymentReceived",
        payload: {
          loanId: result.loan.id,
          repaymentId: result.repayment.id,
          paymentSessionId: result.paymentSession?.id,
          checkoutUrl: result.paymentSession?.checkoutUrl,
          webhookUrl: result.paymentSession?.webhookUrl,
          webhookSecret: result.paymentSession?.webhookSecret,
          amount: result.repayment.amount.toString(),
          outstandingAfter: result.outstandingAfter,
          paidAt: result.repayment.paidAt.toISOString(),
          successUrl: result.paymentSession?.successUrl,
          cancelUrl: result.paymentSession?.cancelUrl,
        },
      })
      .catch((error) => {
        console.error(
          `Failed to enqueue payment webhook for loan ${result.loan.id}:`,
          error,
        );
      });

    return result;
  }
}

export default new LoanService();

import { ForbiddenError, NotFoundError, ValidationError } from "../config/errors";
import { contracts } from "../config/contracts";
import contractService from "./contract.service";
import { prisma } from "./database.service";
import { Prisma } from "@prisma/client";

const MIN_COLLATERAL_RATIO = new Prisma.Decimal("1.5");
const VALID_LOAN_STATUSES = new Set(["PENDING", "ACTIVE", "REPAID", "DEFAULTED"]);
const ZERO = new Prisma.Decimal("0");

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
}

function parsePositiveDecimal(value: number | string | undefined, fieldName: string): Prisma.Decimal {
    let parsed: Prisma.Decimal;
    try {
        parsed = new Prisma.Decimal(value as string | number);
    } catch {
        throw new ValidationError(`${fieldName} must be a positive number`);
    }
    if (!parsed.isFinite() || parsed.lte(ZERO)) {
        throw new ValidationError(`${fieldName} must be a positive number`);
    }
    return parsed;
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
            throw new ForbiddenError("Only the borrower or lender can create this loan");
        }

        const amount = parsePositiveDecimal(payload.amount, "amount");
        const collateralAmt = parsePositiveDecimal(payload.collateralAmt, "collateralAmt");
        const collateralRatio = collateralAmt.div(amount);
        if (collateralRatio.lt(MIN_COLLATERAL_RATIO)) {
            throw new ValidationError(
                `Collateral ratio must be at least ${MIN_COLLATERAL_RATIO.toFixed(2)}`
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

        const xdr = await contractService.buildContractInvokeXDR(
            loanContractId,
            "issue_loan",
            [
                borrowerId,
                lenderId,
                amount.toString(),
                collateralAmt.toString(),
                payload.assetCode || "USDC",
                payload.escrowAddress || null,
            ]
        );

        const loan = await db.loan.create({
            data: {
                borrowerId,
                lenderId,
                amount: amount.toString(),
                collateralAmt: collateralAmt.toString(),
                assetCode: payload.assetCode || "USDC",
                escrowAddress: payload.escrowAddress || null,
                status: "PENDING",
            },
        });

        return {
            loanId: loan.id,
            xdr,
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
            throw new ValidationError("Invalid status. Use PENDING, ACTIVE, REPAID, or DEFAULTED");
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
        return db.$transaction(async (tx: any) => {
            const loan = await tx.loan.findUnique({
                where: { id: loanId },
                include: { repayments: true, borrower: true, lender: true },
            });
            if (!loan) {
                throw new NotFoundError("Loan not found");
            }
            if (requestingUserId !== loan.borrowerId && requestingUserId !== loan.lenderId) {
                throw new ForbiddenError("Only the borrower or lender can record repayments");
            }
            if (loan.status === "DEFAULTED") {
                throw new ValidationError("Cannot record repayment for a defaulted loan");
            }

            const totalRepaid = loan.repayments.reduce(
                (sum: Prisma.Decimal, repayment: { amount: string | number }) =>
                    sum.plus(new Prisma.Decimal(repayment.amount.toString())),
                ZERO
            );
            const outstandingBefore = new Prisma.Decimal(loan.amount.toString()).minus(totalRepaid);
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
                outstandingBefore: outstandingBefore.toString(),
                outstandingAfter: outstandingAfter.toString(),
                fullyRepaid: outstandingAfter.eq(ZERO),
                loan: updatedLoan,
            };
        }, { isolationLevel: "Serializable" });
    }
}

export default new LoanService();

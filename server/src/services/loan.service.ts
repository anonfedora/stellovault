import { NotFoundError, ValidationError } from "../config/errors";
import { contracts } from "../config/contracts";
import contractService from "./contract.service";
import { prisma } from "./database.service";

const MIN_COLLATERAL_RATIO = 1.5;
const VALID_LOAN_STATUSES = new Set(["PENDING", "ACTIVE", "REPAID", "DEFAULTED"]);
const EPSILON = 1e-9;

type LoanStatus = "PENDING" | "ACTIVE" | "REPAID" | "DEFAULTED";

interface IssueLoanRequest {
    borrowerId?: string;
    lenderId?: string;
    amount?: number | string;
    assetCode?: string;
    collateralAmt?: number | string;
    escrowAddress?: string;
}

interface RecordRepaymentRequest {
    loanId?: string;
    amount?: number | string;
    paidAt?: string | Date;
}

function parsePositiveAmount(value: number | string | undefined, fieldName: string): number {
    const parsed = Number(value);
    if (!Number.isFinite(parsed) || parsed <= 0) {
        throw new ValidationError(`${fieldName} must be a positive number`);
    }
    return parsed;
}

export class LoanService {
    async issueLoan(payload: IssueLoanRequest) {
        const borrowerId = payload.borrowerId?.trim();
        const lenderId = payload.lenderId?.trim();
        if (!borrowerId) {
            throw new ValidationError("borrowerId is required");
        }
        if (!lenderId) {
            throw new ValidationError("lenderId is required");
        }

        const amount = parsePositiveAmount(payload.amount, "amount");
        const collateralAmt = parsePositiveAmount(payload.collateralAmt, "collateralAmt");
        const collateralRatio = collateralAmt / amount;
        if (collateralRatio < MIN_COLLATERAL_RATIO) {
            throw new ValidationError(
                `Collateral ratio must be at least ${MIN_COLLATERAL_RATIO.toFixed(2)}`
            );
        }

        const db: any = prisma;
        const users = await db.user.findMany({
            where: { id: { in: [borrowerId, lenderId] } },
            select: { id: true },
        });
        if (users.length !== 2) {
            throw new ValidationError("borrowerId or lenderId does not exist");
        }

        const xdr = await contractService.buildContractInvokeXDR(
            contracts.loan || "LOAN_CONTRACT_ID_NOT_SET",
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
        const db: any = prisma;
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
        const db: any = prisma;
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
        const loanId = payload.loanId?.trim();
        if (!loanId) {
            throw new ValidationError("loanId is required");
        }

        const amount = parsePositiveAmount(payload.amount, "amount");
        let paidAt: Date | undefined;
        if (payload.paidAt) {
            paidAt = new Date(payload.paidAt);
            if (Number.isNaN(paidAt.getTime())) {
                throw new ValidationError("paidAt must be a valid date");
            }
        }

        const db: any = prisma;
        return db.$transaction(async (tx: any) => {
            const loan = await tx.loan.findUnique({
                where: { id: loanId },
                include: { repayments: true, borrower: true, lender: true },
            });
            if (!loan) {
                throw new NotFoundError("Loan not found");
            }
            if (loan.status === "DEFAULTED") {
                throw new ValidationError("Cannot record repayment for a defaulted loan");
            }

            const totalRepaid = loan.repayments.reduce(
                (sum: number, repayment: { amount: string | number }) => sum + Number(repayment.amount),
                0
            );
            const outstandingBefore = Number(loan.amount) - totalRepaid;
            if (outstandingBefore <= EPSILON) {
                throw new ValidationError("Loan is already fully repaid");
            }
            if (amount - outstandingBefore > EPSILON) {
                throw new ValidationError("Repayment exceeds outstanding balance");
            }

            const repayment = await tx.repayment.create({
                data: {
                    loanId,
                    amount: amount.toString(),
                    ...(paidAt ? { paidAt } : {}),
                },
            });

            const outstandingAfter = Math.max(0, outstandingBefore - amount);
            let nextStatus: LoanStatus = loan.status;
            if (outstandingAfter <= EPSILON) {
                nextStatus = "REPAID";
            } else if (loan.status === "PENDING") {
                nextStatus = "ACTIVE";
            }

            const updatedLoan =
                nextStatus === loan.status
                    ? loan
                    : await tx.loan.update({
                        where: { id: loanId },
                        data: { status: nextStatus },
                        include: { repayments: true, borrower: true, lender: true },
                    });

            return {
                repayment,
                outstandingBefore,
                outstandingAfter,
                fullyRepaid: outstandingAfter <= EPSILON,
                loan: updatedLoan,
            };
        });
    }
}

export default new LoanService();

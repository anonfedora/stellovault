-- AlterTable
ALTER TABLE "Loan" ADD COLUMN "lenderId" TEXT;

-- CreateTable
CREATE TABLE "Repayment" (
    "id" TEXT NOT NULL,
    "loanId" TEXT NOT NULL,
    "amount" DECIMAL(65,30) NOT NULL,
    "paidAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Repayment_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE INDEX "Loan_borrowerId_idx" ON "Loan"("borrowerId");

-- CreateIndex
CREATE INDEX "Loan_lenderId_idx" ON "Loan"("lenderId");

-- CreateIndex
CREATE INDEX "Loan_status_idx" ON "Loan"("status");

-- CreateIndex
CREATE INDEX "Repayment_loanId_idx" ON "Repayment"("loanId");

-- AddForeignKey
ALTER TABLE "Loan" ADD CONSTRAINT "Loan_lenderId_fkey" FOREIGN KEY ("lenderId") REFERENCES "User"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Repayment" ADD CONSTRAINT "Repayment_loanId_fkey" FOREIGN KEY ("loanId") REFERENCES "Loan"("id") ON DELETE CASCADE ON UPDATE CASCADE;

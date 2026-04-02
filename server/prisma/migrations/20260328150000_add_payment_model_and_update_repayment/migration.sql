-- Add PaymentStatus column to Repayment table
ALTER TABLE "Repayment" ADD COLUMN "status" "PaymentStatus" NOT NULL DEFAULT E'PENDING';

-- Create Payment table
CREATE TABLE "Payment" (
    "id" TEXT NOT NULL,
    "loanId" TEXT NOT NULL,
    "amount" DECIMAL(20,7) NOT NULL,
    "status" "PaymentStatus" NOT NULL DEFAULT E'PENDING',
    "txHash" TEXT,
    "errorMessage" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "completedAt" TIMESTAMP(3),

    CONSTRAINT "Payment_pkey" PRIMARY KEY ("id")
);

-- Create indexes
CREATE INDEX "Repayment_status_idx" ON "Repayment"("status");
CREATE INDEX "Payment_loanId_idx" ON "Payment"("loanId");
CREATE INDEX "Payment_status_idx" ON "Payment"("status");

-- Add foreign key constraint
ALTER TABLE "Payment" ADD CONSTRAINT "Payment_loanId_fkey" FOREIGN KEY ("loanId") REFERENCES "Loan"("id") ON DELETE CASCADE ON UPDATE CASCADE;

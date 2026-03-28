CREATE TYPE "PaymentSessionStatus" AS ENUM ('PENDING', 'COMPLETED', 'CANCELLED', 'EXPIRED');

CREATE TABLE "PaymentSession" (
    "id" TEXT NOT NULL,
    "loanId" TEXT NOT NULL,
    "repaymentId" TEXT,
    "sessionToken" TEXT NOT NULL,
    "checkoutUrl" TEXT NOT NULL,
    "successUrl" TEXT NOT NULL,
    "cancelUrl" TEXT NOT NULL,
    "webhookUrl" TEXT,
    "webhookSecret" TEXT,
    "status" "PaymentSessionStatus" NOT NULL DEFAULT 'PENDING',
    "completedAt" TIMESTAMP(3),
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "PaymentSession_pkey" PRIMARY KEY ("id")
);

CREATE UNIQUE INDEX "PaymentSession_repaymentId_key" ON "PaymentSession"("repaymentId");
CREATE UNIQUE INDEX "PaymentSession_sessionToken_key" ON "PaymentSession"("sessionToken");
CREATE INDEX "PaymentSession_loanId_idx" ON "PaymentSession"("loanId");
CREATE INDEX "PaymentSession_status_idx" ON "PaymentSession"("status");

ALTER TABLE "PaymentSession" ADD CONSTRAINT "PaymentSession_loanId_fkey" FOREIGN KEY ("loanId") REFERENCES "Loan"("id") ON DELETE CASCADE ON UPDATE CASCADE;
ALTER TABLE "PaymentSession" ADD CONSTRAINT "PaymentSession_repaymentId_fkey" FOREIGN KEY ("repaymentId") REFERENCES "Repayment"("id") ON DELETE SET NULL ON UPDATE CASCADE;

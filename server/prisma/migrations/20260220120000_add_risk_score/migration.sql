-- CreateTable
CREATE TABLE "RiskScore" (
    "id" TEXT NOT NULL,
    "wallet" TEXT NOT NULL,
    "score" INTEGER NOT NULL,
    "grade" TEXT NOT NULL,
    "components" JSONB NOT NULL,
    "computedAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "RiskScore_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE INDEX "RiskScore_wallet_computedAt_idx" ON "RiskScore"("wallet", "computedAt");

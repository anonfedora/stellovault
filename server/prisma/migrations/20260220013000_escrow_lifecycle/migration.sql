-- CreateEnum
CREATE TYPE "EscrowStatus" AS ENUM ('PENDING', 'ACTIVE', 'COMPLETED', 'DISPUTED', 'EXPIRED');

-- CreateTable
CREATE TABLE "Escrow" (
    "id" TEXT NOT NULL,
    "buyerId" TEXT NOT NULL,
    "sellerId" TEXT NOT NULL,
    "amount" DECIMAL(65,30) NOT NULL,
    "assetCode" TEXT NOT NULL DEFAULT 'USDC',
    "status" "EscrowStatus" NOT NULL DEFAULT 'PENDING',
    "stellarTxHash" TEXT,
    "expiresAt" TIMESTAMP(3) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Escrow_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE INDEX "Escrow_buyerId_idx" ON "Escrow"("buyerId");

-- CreateIndex
CREATE INDEX "Escrow_sellerId_idx" ON "Escrow"("sellerId");

-- CreateIndex
CREATE INDEX "Escrow_status_idx" ON "Escrow"("status");

-- CreateIndex
CREATE INDEX "Escrow_expiresAt_idx" ON "Escrow"("expiresAt");

-- AddForeignKey
ALTER TABLE "Escrow" ADD CONSTRAINT "Escrow_buyerId_fkey" FOREIGN KEY ("buyerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Escrow" ADD CONSTRAINT "Escrow_sellerId_fkey" FOREIGN KEY ("sellerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

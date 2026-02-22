-- CreateEnum
CREATE TYPE "Role" AS ENUM ('USER', 'MERCHANT', 'ADMIN');

-- CreateEnum
CREATE TYPE "ChallengePurpose" AS ENUM ('LOGIN', 'LINK_WALLET');

-- CreateEnum
CREATE TYPE "EscrowStatus" AS ENUM ('PENDING', 'ACTIVE', 'COMPLETED', 'DISPUTED', 'EXPIRED');

-- CreateEnum
CREATE TYPE "CollateralStatus" AS ENUM ('LOCKED', 'RELEASED', 'LIQUIDATED');

-- CreateEnum
CREATE TYPE "LoanStatus" AS ENUM ('PENDING', 'ACTIVE', 'REPAID', 'DEFAULTED');

-- CreateEnum
CREATE TYPE "GovernanceStatus" AS ENUM ('OPEN', 'PASSED', 'REJECTED', 'EXECUTED');

-- CreateTable
CREATE TABLE "User" (
    "id" TEXT NOT NULL,
    "stellarAddress" TEXT NOT NULL,
    "name" TEXT,
    "role" "Role" NOT NULL DEFAULT 'USER',
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "User_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Wallet" (
    "id" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "address" TEXT NOT NULL,
    "isPrimary" BOOLEAN NOT NULL DEFAULT false,
    "label" TEXT,
    "verifiedAt" TIMESTAMP(3),
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Wallet_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "WalletChallenge" (
    "id" TEXT NOT NULL,
    "userId" TEXT,
    "walletAddress" TEXT NOT NULL,
    "nonce" TEXT NOT NULL,
    "purpose" "ChallengePurpose" NOT NULL,
    "expiresAt" TIMESTAMP(3) NOT NULL,
    "usedAt" TIMESTAMP(3),
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "WalletChallenge_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Session" (
    "id" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "jti" TEXT NOT NULL,
    "revokedAt" TIMESTAMP(3),
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Session_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Escrow" (
    "id" TEXT NOT NULL,
    "buyerId" TEXT NOT NULL,
    "sellerId" TEXT NOT NULL,
    "amount" DECIMAL(20,7) NOT NULL,
    "assetCode" TEXT NOT NULL DEFAULT 'USDC',
    "status" "EscrowStatus" NOT NULL DEFAULT 'PENDING',
    "stellarTxHash" TEXT,
    "expiresAt" TIMESTAMP(3) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Escrow_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Collateral" (
    "id" TEXT NOT NULL,
    "escrowId" TEXT NOT NULL,
    "assetCode" TEXT NOT NULL,
    "amount" DECIMAL(20,7) NOT NULL,
    "metadataHash" TEXT,
    "status" "CollateralStatus" NOT NULL DEFAULT 'LOCKED',
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Collateral_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Loan" (
    "id" TEXT NOT NULL,
    "borrowerId" TEXT NOT NULL,
    "lenderId" TEXT,
    "amount" DECIMAL(20,7) NOT NULL,
    "assetCode" TEXT NOT NULL DEFAULT 'USDC',
    "interestRate" DECIMAL(5,4) NOT NULL,
    "status" "LoanStatus" NOT NULL DEFAULT 'PENDING',
    "dueDate" TIMESTAMP(3),
    "collateralId" TEXT,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "Loan_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Repayment" (
    "id" TEXT NOT NULL,
    "loanId" TEXT NOT NULL,
    "amount" DECIMAL(20,7) NOT NULL,
    "paidAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Repayment_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "OracleEvent" (
    "id" TEXT NOT NULL,
    "escrowId" TEXT NOT NULL,
    "oracleAddress" TEXT NOT NULL,
    "confirmationType" TEXT NOT NULL,
    "signature" TEXT NOT NULL,
    "confirmedAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "OracleEvent_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "GovernanceProposal" (
    "id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "proposerId" TEXT NOT NULL,
    "status" "GovernanceStatus" NOT NULL DEFAULT 'OPEN',
    "endsAt" TIMESTAMP(3) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "GovernanceProposal_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "GovernanceVote" (
    "id" TEXT NOT NULL,
    "proposalId" TEXT NOT NULL,
    "voterAddress" TEXT NOT NULL,
    "vote" TEXT NOT NULL,
    "weight" DECIMAL(20,7) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "GovernanceVote_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "RiskScore" (
    "id" TEXT NOT NULL,
    "walletAddress" TEXT NOT NULL,
    "score" INTEGER NOT NULL,
    "components" JSONB NOT NULL,
    "recordedAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "RiskScore_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "User_stellarAddress_key" ON "User"("stellarAddress");

-- CreateIndex
CREATE UNIQUE INDEX "Wallet_address_key" ON "Wallet"("address");

-- CreateIndex
CREATE INDEX "Wallet_userId_idx" ON "Wallet"("userId");

-- CreateIndex
CREATE INDEX "WalletChallenge_walletAddress_purpose_idx" ON "WalletChallenge"("walletAddress", "purpose");

-- CreateIndex
CREATE INDEX "WalletChallenge_userId_purpose_idx" ON "WalletChallenge"("userId", "purpose");

-- CreateIndex
CREATE UNIQUE INDEX "WalletChallenge_walletAddress_nonce_purpose_key" ON "WalletChallenge"("walletAddress", "nonce", "purpose");

-- CreateIndex
CREATE UNIQUE INDEX "Session_jti_key" ON "Session"("jti");

-- CreateIndex
CREATE INDEX "Session_userId_idx" ON "Session"("userId");

-- CreateIndex
CREATE INDEX "Escrow_buyerId_idx" ON "Escrow"("buyerId");

-- CreateIndex
CREATE INDEX "Escrow_sellerId_idx" ON "Escrow"("sellerId");

-- CreateIndex
CREATE INDEX "Escrow_status_idx" ON "Escrow"("status");

-- CreateIndex
CREATE INDEX "Escrow_expiresAt_idx" ON "Escrow"("expiresAt");

-- CreateIndex
CREATE INDEX "Collateral_escrowId_idx" ON "Collateral"("escrowId");

-- CreateIndex
CREATE INDEX "Collateral_metadataHash_idx" ON "Collateral"("metadataHash");

-- CreateIndex
CREATE INDEX "Loan_borrowerId_idx" ON "Loan"("borrowerId");

-- CreateIndex
CREATE INDEX "Loan_lenderId_idx" ON "Loan"("lenderId");

-- CreateIndex
CREATE INDEX "Loan_status_idx" ON "Loan"("status");

-- CreateIndex
CREATE INDEX "Loan_collateralId_idx" ON "Loan"("collateralId");

-- CreateIndex
CREATE INDEX "Repayment_loanId_idx" ON "Repayment"("loanId");

-- CreateIndex
CREATE INDEX "OracleEvent_escrowId_idx" ON "OracleEvent"("escrowId");

-- CreateIndex
CREATE INDEX "OracleEvent_oracleAddress_idx" ON "OracleEvent"("oracleAddress");

-- CreateIndex
CREATE INDEX "GovernanceProposal_proposerId_idx" ON "GovernanceProposal"("proposerId");

-- CreateIndex
CREATE INDEX "GovernanceProposal_status_idx" ON "GovernanceProposal"("status");

-- CreateIndex
CREATE INDEX "GovernanceVote_proposalId_idx" ON "GovernanceVote"("proposalId");

-- CreateIndex
CREATE UNIQUE INDEX "GovernanceVote_proposalId_voterAddress_key" ON "GovernanceVote"("proposalId", "voterAddress");

-- CreateIndex
CREATE INDEX "RiskScore_walletAddress_idx" ON "RiskScore"("walletAddress");

-- CreateIndex
CREATE INDEX "RiskScore_recordedAt_idx" ON "RiskScore"("recordedAt");

-- AddForeignKey
ALTER TABLE "Wallet" ADD CONSTRAINT "Wallet_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "WalletChallenge" ADD CONSTRAINT "WalletChallenge_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Session" ADD CONSTRAINT "Session_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Escrow" ADD CONSTRAINT "Escrow_buyerId_fkey" FOREIGN KEY ("buyerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Escrow" ADD CONSTRAINT "Escrow_sellerId_fkey" FOREIGN KEY ("sellerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Collateral" ADD CONSTRAINT "Collateral_escrowId_fkey" FOREIGN KEY ("escrowId") REFERENCES "Escrow"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Loan" ADD CONSTRAINT "Loan_borrowerId_fkey" FOREIGN KEY ("borrowerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Loan" ADD CONSTRAINT "Loan_lenderId_fkey" FOREIGN KEY ("lenderId") REFERENCES "User"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Loan" ADD CONSTRAINT "Loan_collateralId_fkey" FOREIGN KEY ("collateralId") REFERENCES "Collateral"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Repayment" ADD CONSTRAINT "Repayment_loanId_fkey" FOREIGN KEY ("loanId") REFERENCES "Loan"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "OracleEvent" ADD CONSTRAINT "OracleEvent_escrowId_fkey" FOREIGN KEY ("escrowId") REFERENCES "Escrow"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "GovernanceProposal" ADD CONSTRAINT "GovernanceProposal_proposerId_fkey" FOREIGN KEY ("proposerId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "GovernanceVote" ADD CONSTRAINT "GovernanceVote_proposalId_fkey" FOREIGN KEY ("proposalId") REFERENCES "GovernanceProposal"("id") ON DELETE CASCADE ON UPDATE CASCADE;

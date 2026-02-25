-- Migrate existing Escrow status values to new enum values
-- Mapping: PENDING -> PENDING, ACTIVE -> FUNDED, COMPLETED -> RELEASED, DISPUTED -> DISPUTED, EXPIRED -> CANCELLED
UPDATE "Escrow" SET status = 'FUNDED' WHERE status = 'ACTIVE';
UPDATE "Escrow" SET status = 'RELEASED' WHERE status = 'COMPLETED';
UPDATE "Escrow" SET status = 'CANCELLED' WHERE status = 'EXPIRED';

-- Alter the enum type
-- First, change the column to text
ALTER TABLE "Escrow" ALTER COLUMN status TYPE TEXT;

-- Drop the old enum
DROP TYPE "EscrowStatus";

-- Create the new enum with correct values
CREATE TYPE "EscrowStatus" AS ENUM ('PENDING', 'FUNDED', 'RELEASED', 'REFUNDED', 'DISPUTED', 'CANCELLED');

-- Change the column back to the enum type
ALTER TABLE "Escrow" ALTER COLUMN status TYPE "EscrowStatus" USING status::"EscrowStatus";

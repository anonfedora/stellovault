-- Custom Constraint: Only allow one 'isPrimary = true' per userId
CREATE UNIQUE INDEX "Wallet_userId_isPrimary_true_idx" 
ON "Wallet"("userId") 
WHERE "isPrimary" = TRUE;
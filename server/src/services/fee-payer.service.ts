import { Transaction, Horizon } from "@stellar/stellar-sdk";
import vaultService from "./vault.service";
import { env } from "../config/env";
import { prisma } from "../config/prisma";

/**
 * FeePayerService manages multiple fee-payer accounts to parallelize
 * transaction submissions and ensure high availability.
 */
export class FeePayerService {
    private server: Horizon.Server;

    constructor() {
        this.server = new Horizon.Server(env.stellar.horizonUrl);
    }

    /**
     * Retrieves the best fee-payer account to use for a new transaction.
     * Implements a round-robin or least-recently-used (LRU) selection mechanism
     * to avoid sequence number collisions and parallelize submissions.
     * 
     * @returns The public key of the selected fee-payer account.
     */
    async getFeePayer(): Promise<string> {
        // 1. Try to find available accounts in the database.
        try {
            const feePayer = await prisma.feePayer.findFirst({
                where: { isPaused: false },
                orderBy: { lastUsedAt: "asc" }, // Pick the one that was used longest ago (round-robin)
            });

            if (feePayer) {
                // Update lastUsedAt to mark it as the most recently used.
                await prisma.feePayer.update({
                    where: { id: feePayer.id },
                    data: { lastUsedAt: new Date() },
                });
                return feePayer.publicKey;
            }
        } catch (err) {
            console.warn("[FeePayerService] Database access failed, falling back to environment config.", err);
        }

        // 2. Fallback to the environment-configured fee payer.
        return env.feePayer.publicKey;
    }

    /**
     * Signs a transaction using the specified fee-payer's key from Vault.
     * 
     * @param publicKey The public key of the fee-payer.
     * @param transaction The transaction to be signed.
     * @returns The signed transaction.
     */
    async sign(publicKey: string, transaction: Transaction): Promise<Transaction> {
        return await vaultService.signTransaction(publicKey, transaction);
    }

    /**
     * Refreshes the pool of fee-payer accounts from the database.
     * Useful if new accounts were added via manual migration or rebalancing script.
     */
    async refreshAccountPool(): Promise<void> {
        // In this implementation, we always fetch from the DB in getFeePayer,
        // so this might just be a notification mechanism or internal cache refresh.
        console.log("[FeePayerService] Refreshing account pool...");
    }

    /**
     * Get the current balance of all fee payers combined.
     */
    async getTotalPoolBalance(): Promise<number> {
        try {
            const feePayers = await prisma.feePayer.findMany({
                where: { isPaused: false },
            });

            let total = 0;
            for (const fp of feePayers) {
                const balance = await this.server.loadAccount(fp.publicKey);
                const native = balance.balances.find(
                    (b: Horizon.HorizonApi.BalanceLine) => b.asset_type === "native"
                );
                total += parseFloat(native?.balance ?? "0");
            }

            return total;
        } catch (err) {
            console.error("[FeePayerService] Could not calculate total pool balance.", err);
            return 0;
        }
    }
}

export default new FeePayerService();

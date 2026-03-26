import {
    Horizon,
    TransactionBuilder,
    Operation,
    Asset,
    Keypair,
    BASE_FEE,
} from "@stellar/stellar-sdk";
import { env } from "../config/env";
import { prisma } from "../config/prisma";
import feePayerService from "./fee-payer.service";

/**
 * RebalancingService automates the funding of fee-payer accounts from
 * a "cold" treasury wallet to ensure they have enough gas for user transactions.
 */
export class RebalancingService {
    private server: Horizon.Server;

    constructor() {
        this.server = new Horizon.Server(env.stellar.horizonUrl);
    }

    /**
     * Checks all fee-payer accounts and tops up those that are below the
     * minimum balance threshold.
     */
    async rebalanceAll(): Promise<void> {
        console.log("[RebalancingService] Starting rebalancing check...");

        try {
            const feePayers = await prisma.feePayer.findMany({
                where: { isPaused: false },
            });

            for (const fp of feePayers) {
                await this.rebalanceAccount(fp.publicKey);
            }

            // Also check the total pool balance for alerting
            const totalBalance = await feePayerService.getTotalPoolBalance();
            if (totalBalance < env.feePayer.minBalance * feePayers.length * 0.5) {
                console.error(`[ALERT] Total fee-payer pool balance is low: ${totalBalance} XLM`);
                // TODO: Integrate with monitoring service (e.g., PagerDuty, email, Slack)
            }

        } catch (err) {
            console.error("[RebalancingService] Rebalancing failed.", err);
        }
    }

    /**
     * Tops up a single fee-payer account if its balance is low.
     * 
     * @param publicKey The public key of the account to check and top up.
     */
    async rebalanceAccount(publicKey: string): Promise<void> {
        try {
            const account = await this.server.loadAccount(publicKey);
            const native = account.balances.find(
                (b: Horizon.HorizonApi.BalanceLine) => b.asset_type === "native"
            );
            const balance = parseFloat(native?.balance ?? "0");

            if (balance < env.feePayer.minBalance) {
                const topUpAmount = (env.feePayer.minBalance * 2 - balance).toString();
                console.log(`[RebalancingService] Topping up ${publicKey} with ${topUpAmount} XLM...`);

                await this.fundFromTreasury(publicKey, topUpAmount);
            }
        } catch (err) {
            console.error(`[RebalancingService] Could not rebalance account ${publicKey}:`, err);
        }
    }

    /**
     * Executes a payment from the treasury wallet to a fee-payer account.
     * 
     * @param to The recipient fee-payer public key.
     * @param amount The amount of XLM to send.
     */
    private async fundFromTreasury(to: string, amount: string): Promise<void> {
        if (!env.treasury.secretKey) {
            console.warn("[RebalancingService] No TREASURY_SECRET configured. Skipping funding.");
            return;
        }

        const treasuryKeypair = Keypair.fromSecret(env.treasury.secretKey);
        const treasuryAccount = await this.server.loadAccount(treasuryKeypair.publicKey());

        const tx = new TransactionBuilder(treasuryAccount, {
            fee: BASE_FEE,
            networkPassphrase: env.stellar.networkPassphrase,
        })
            .addOperation(
                Operation.payment({
                    destination: to,
                    asset: Asset.native(),
                    amount,
                }),
            )
            .setTimeout(30)
            .build();

        tx.sign(treasuryKeypair);

        try {
            const response = await this.server.submitTransaction(tx);
            console.log(`[RebalancingService] Successfully funded ${to}. Tx hash: ${response.hash}`);
        } catch (err) {
            console.error(`[RebalancingService] Failed to fund ${to}:`, err);
        }
    }
}

export default new RebalancingService();

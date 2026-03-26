import dotenv from "dotenv";
dotenv.config();

import rebalancingService from "../services/rebalancing.service";

/**
 * CLI script to manually trigger rebalancing of fee-payer accounts.
 * Run with: npx ts-node src/scripts/rebalance.ts
 */
async function main() {
    console.log("--- Fee Payer Rebalancing Script ---");
    try {
        await rebalancingService.rebalanceAll();
        console.log("Rebalancing completed successfully.");
        process.exit(0);
    } catch (err) {
        console.error("Rebalancing failed:", err);
        process.exit(1);
    }
}

main();

/**
 * Integration smoke-test against Stellar testnet.
 *
 * Prerequisites:
 *   1. Create a .env with funded testnet keys:
 *        FEE_PAYER_SECRET=S...
 *        FEE_PAYER_PUBLIC=G...
 *   2. Fund both the fee payer and a test user via Friendbot:
 *        curl "https://friendbot.stellar.org/?addr=<PUBLIC_KEY>"
 *   3. (Optional) Deploy a contract and set its ID below.
 *
 * Run:
 *   npx ts-node-dev src/tests/integration/smoke.test.ts
 */

import dotenv from "dotenv";
dotenv.config();

import {
    Keypair,
    TransactionBuilder,
    Networks,
    Transaction,
    xdr,
    nativeToScVal,
} from "@stellar/stellar-sdk";
import { BlockchainService } from "../../services/blockchain.service";
import { ContractService } from "../../services/contract.service";
import { env } from "../../config/env";

const DIVIDER = "─".repeat(60);

function log(label: string, value: unknown) {
    console.log(`  ${label}:`, value);
}

async function testBlockchainService() {
    console.log("\n" + DIVIDER);
    console.log("BlockchainService – integration tests");
    console.log(DIVIDER);

    const svc = new BlockchainService();

    /* ---- getAccountBalance ---- */
    console.log("\n[1] getAccountBalance (native XLM)");
    const balance = await svc.getAccountBalance(env.feePayer.publicKey, "XLM");
    log("Fee payer XLM balance", balance);
    console.assert(
        parseFloat(balance) > 0,
        "Fee payer should have a positive balance",
    );

    /* ---- buildNativePayment ---- */
    console.log("\n[2] buildNativePayment");
    const testUser = Keypair.random();
    log("Recipient (random, unfunded)", testUser.publicKey());

    const xdrString = await svc.buildNativePayment(
        env.feePayer.publicKey,
        testUser.publicKey(),
        "1",
    );
    log("XDR length", xdrString.length);

    const decoded = TransactionBuilder.fromXDR(
        xdrString,
       env.stellar.networkPassphrase,
    ) as Transaction;
    log("Decoded tx source", decoded.source);
    log("Operation count", decoded.operations.length);
    log("Signature count", decoded.signatures.length);

    console.assert(
        decoded.source === env.feePayer.publicKey,
        "Outer source should be fee payer",
    );
    console.assert(
        decoded.signatures.length >= 1,
        "Should have at least 1 signature",
    );

    console.log("  ✓ XDR is valid and decodable\n");
}

async function testContractService() {
    console.log(DIVIDER);
    console.log("ContractService – integration tests");
    console.log(DIVIDER);

    const svc = new ContractService();

    /* ---- simulateCall (requires a deployed contract) ---- */
    const contractId = process.env.LOAN_CONTRACT_ID;
    if (!contractId) {
        console.log(
            "\n  ⚠  Skipping contract tests — no LOAN_CONTRACT_ID set.",
        );
        console.log(
            "     Deploy a contract, add its ID to .env, and re-run.\n",
        );
        return;
    }

    console.log("\n[3] simulateCall");
    try {
        const result = await svc.simulateCall(contractId, "get_status", []);
        log("Simulation result (decoded)", result);
        console.log("  ✓ simulateCall succeeded\n");
    } catch (err: any) {
        console.log("  ✗ simulateCall failed:", err.message);
        console.log(
            "    (expected if the contract doesn't have a get_status method)\n",
        );
    }

    /* ---- buildContractInvokeXDR ---- */
    console.log("[4] buildContractInvokeXDR");
    const user = Keypair.random();
    try {
        const invokeXDR = await svc.buildContractInvokeXDR(
            contractId,
            "get_status",
            [],
            user.publicKey(),
        );
        log("XDR length", invokeXDR.length);

        const decoded = TransactionBuilder.fromXDR(
            invokeXDR,
            Networks.TESTNET,
        ) as Transaction;
        log("Decoded tx source (fee payer)", decoded.source);
        log("Signatures", decoded.signatures.length);

        console.assert(
            decoded.source === env.feePayer.publicKey,
            "Fee payer should be the outer source",
        );
        console.log("  ✓ buildContractInvokeXDR succeeded\n");
    } catch (err: any) {
        console.log("  ✗ buildContractInvokeXDR failed:", err.message, "\n");
    }
}

async function main() {
    console.log("\n═══════════════════════════════════════════════════════════");
    console.log("  StelloVault – Service Integration Smoke Test");
    console.log("═══════════════════════════════════════════════════════════");

    if (!env.feePayer.publicKey || !env.feePayer.secretKey) {
        console.error(
            "\n  ✗ FEE_PAYER_PUBLIC and FEE_PAYER_SECRET must be set in .env",
        );
        console.error("    Generate a testnet keypair:");
        console.error(
            "      const kp = Keypair.random(); console.log(kp.publicKey(), kp.secret());",
        );
        console.error(
            '    Fund it: curl "https://friendbot.stellar.org/?addr=<PUBLIC>"',
        );
        process.exit(1);
    }

    try {
        await testBlockchainService();
        await testContractService();
    } catch (err) {
        console.error("\n  ✗ Unexpected error:", err);
        process.exit(1);
    }

    console.log(DIVIDER);
    console.log("  Done.");
    console.log(DIVIDER + "\n");
}

main();

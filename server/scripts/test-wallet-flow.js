#!/usr/bin/env node

const path = require("path");
const dotenv = require("dotenv");
const jwt = require("jsonwebtoken");
const { randomUUID } = require("crypto");
const { Keypair } = require("@stellar/stellar-sdk");
const { PrismaClient } = require("@prisma/client");

dotenv.config({ path: path.join(__dirname, "..", ".env") });

const API_BASE = process.env.API_BASE || "http://localhost:3001/api";
const TEST_USER_ID = process.env.TEST_USER_ID || randomUUID();
const JWT_ACCESS_SECRET = process.env.JWT_ACCESS_SECRET || "change-me-in-prod";

const prisma = new PrismaClient();

async function request(method, endpoint, token, body) {
    const res = await fetch(`${API_BASE}${endpoint}`, {
        method,
        headers: {
            ...(body ? { "Content-Type": "application/json" } : {}),
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: body ? JSON.stringify(body) : undefined,
    });

    const text = await res.text();
    let json = null;
    try {
        json = text ? JSON.parse(text) : null;
    } catch {
        json = null;
    }

    return { status: res.status, body: json, raw: text };
}

function assertStatus(name, actual, expected, payload) {
    if (actual !== expected) {
        throw new Error(
            `${name} expected ${expected}, got ${actual}\nPayload: ${JSON.stringify(payload, null, 2)}`
        );
    }
    console.log(`[PASS] ${name}: ${actual}`);
}

function assertCondition(name, condition, payload) {
    if (!condition) {
        throw new Error(`${name} failed\nPayload: ${JSON.stringify(payload, null, 2)}`);
    }
    console.log(`[PASS] ${name}`);
}

async function run() {
    const wallet1 = Keypair.random();
    const wallet2 = Keypair.random();

    const user = await prisma.user.upsert({
        where: { id: TEST_USER_ID },
        update: {},
        create: {
            id: TEST_USER_ID,
            stellarAddress: wallet1.publicKey(),
        },
    });

    // Keep each run deterministic for the target user.
    await prisma.walletChallenge.deleteMany({ where: { userId: user.id } });
    await prisma.wallet.deleteMany({ where: { userId: user.id } });
    await prisma.user.update({
        where: { id: user.id },
        data: { stellarAddress: wallet1.publicKey() },
    });

    const token = jwt.sign(
        {
            userId: user.id,
            jti: `wallet-test-${Date.now()}`,
            walletAddress: wallet1.publicKey(),
        },
        JWT_ACCESS_SECRET,
        { expiresIn: "1h" }
    );

    const challenge1 = await request("POST", "/wallets/challenge", token, {
        walletAddress: wallet1.publicKey(),
    });
    assertStatus("challenge wallet1", challenge1.status, 200, challenge1.body);
    const message1 = challenge1.body?.data?.message;
    const nonce1 = challenge1.body?.data?.nonce;

    const link1 = await request("POST", "/wallets", token, {
        walletAddress: wallet1.publicKey(),
        nonce: nonce1,
        signature: wallet1.sign(Buffer.from(message1)).toString("base64"),
        label: "Primary",
    });
    assertStatus("link wallet1", link1.status, 201, link1.body);

    const dupChallenge = await request("POST", "/wallets/challenge", token, {
        walletAddress: wallet1.publicKey(),
    });
    assertStatus("duplicate challenge", dupChallenge.status, 200, dupChallenge.body);
    const dupMessage = dupChallenge.body?.data?.message;
    const dupNonce = dupChallenge.body?.data?.nonce;

    const duplicate = await request("POST", "/wallets", token, {
        walletAddress: wallet1.publicKey(),
        nonce: dupNonce,
        signature: wallet1.sign(Buffer.from(dupMessage)).toString("base64"),
    });
    assertStatus("duplicate wallet returns 409", duplicate.status, 409, duplicate.body);

    const challenge2 = await request("POST", "/wallets/challenge", token, {
        walletAddress: wallet2.publicKey(),
    });
    assertStatus("challenge wallet2", challenge2.status, 200, challenge2.body);
    const message2 = challenge2.body?.data?.message;
    const nonce2 = challenge2.body?.data?.nonce;

    const link2 = await request("POST", "/wallets", token, {
        walletAddress: wallet2.publicKey(),
        nonce: nonce2,
        signature: wallet2.sign(Buffer.from(message2)).toString("base64"),
        label: "Secondary",
    });
    assertStatus("link wallet2", link2.status, 201, link2.body);

    const list = await request("GET", "/wallets", token);
    assertStatus("list wallets", list.status, 200, list.body);
    const wallets = list.body?.data || [];
    const wallet1Row = wallets.find((w) => w.address === wallet1.publicKey());
    const wallet2Row = wallets.find((w) => w.address === wallet2.publicKey());
    if (!wallet1Row || !wallet2Row) {
        throw new Error(`Linked wallets not found in list response: ${JSON.stringify(wallets, null, 2)}`);
    }

    const setPrimary = await request("PUT", `/wallets/${wallet2Row.id}/primary`, token);
    assertStatus("set wallet2 primary", setPrimary.status, 200, setPrimary.body);

    const patch = await request("PATCH", `/wallets/${wallet2Row.id}`, token, {
        label: "Secondary updated",
    });
    assertStatus("update wallet label", patch.status, 200, patch.body);

    const delete1 = await request("DELETE", `/wallets/${wallet1Row.id}`, token);
    assertStatus("unlink wallet1", delete1.status, 204, delete1.body);

    const deleteLast = await request("DELETE", `/wallets/${wallet2Row.id}`, token);
    assertStatus("unlink last wallet returns 400", deleteLast.status, 400, deleteLast.body);

    // Race test: same challenge consumed concurrently must allow only one success.
    const raceWallet = Keypair.random();
    const raceChallenge = await request("POST", "/wallets/challenge", token, {
        walletAddress: raceWallet.publicKey(),
    });
    assertStatus("race challenge", raceChallenge.status, 200, raceChallenge.body);
    const raceNonce = raceChallenge.body?.data?.nonce;
    const raceMessage = raceChallenge.body?.data?.message;
    assertCondition(
        "race challenge returns nonce+message",
        typeof raceNonce === "string" && typeof raceMessage === "string",
        raceChallenge.body
    );

    const raceSignature = raceWallet.sign(Buffer.from(raceMessage)).toString("base64");
    const racePayload = {
        walletAddress: raceWallet.publicKey(),
        nonce: raceNonce,
        signature: raceSignature,
        label: "Race Wallet",
    };

    const [race1, race2] = await Promise.all([
        request("POST", "/wallets", token, racePayload),
        request("POST", "/wallets", token, racePayload),
    ]);

    console.log("race resp1:", race1.status, race1.body?.error || "ok");
    console.log("race resp2:", race2.status, race2.body?.error || "ok");

    const raceStatuses = [race1.status, race2.status];
    const raceSuccessCount = raceStatuses.filter((status) => status === 201).length;
    const raceFailureCount = raceStatuses.filter((status) => status >= 400 && status < 500).length;
    assertCondition("race exactly one success", raceSuccessCount === 1, raceStatuses);
    assertCondition("race exactly one 4xx failure", raceFailureCount === 1, raceStatuses);

    const raceRows = await prisma.wallet.findMany({
        where: { userId: user.id, address: raceWallet.publicKey() },
    });
    assertCondition("race creates only one wallet row", raceRows.length === 1, raceRows);

    console.log("\nWallet flow test passed.");
}

run()
    .catch((err) => {
        console.error("\nWallet flow test failed.");
        console.error(err?.stack || err);
        process.exitCode = 1;
    })
    .finally(async () => {
        await prisma.$disconnect();
    });

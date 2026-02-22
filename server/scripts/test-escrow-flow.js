#!/usr/bin/env node

const path = require("path");
const { randomUUID } = require("crypto");
const dotenv = require("dotenv");
const { PrismaClient } = require("@prisma/client");
const { Keypair } = require("@stellar/stellar-sdk");

dotenv.config({ path: path.join(__dirname, "..", ".env") });

const API_BASE = process.env.API_BASE || "http://localhost:3001/api";
const WEBHOOK_SECRET = process.env.WEBHOOK_SECRET || "";
const SKIP_TIMEOUT_TEST = process.env.SKIP_TIMEOUT_TEST === "1";
const TIMEOUT_WAIT_MS = Number(process.env.ESCROW_TIMEOUT_WAIT_MS || 130000);
const prisma = new PrismaClient();
const seededUserIds = new Set();
const createdEscrowIds = new Set();

async function request(method, endpoint, body, extraHeaders = {}) {
    const response = await fetch(`${API_BASE}${endpoint}`, {
        method,
        headers: {
            ...(body ? { "Content-Type": "application/json" } : {}),
            ...extraHeaders,
        },
        body: body ? JSON.stringify(body) : undefined,
    });

    const text = await response.text();
    let json = null;
    try {
        json = text ? JSON.parse(text) : null;
    } catch {
        json = null;
    }

    return { status: response.status, body: json, raw: text };
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

function futureIso(offsetMs) {
    return new Date(Date.now() + offsetMs).toISOString();
}

async function seedUsers() {
    const buyerId = randomUUID();
    const sellerId = randomUUID();
    seededUserIds.add(buyerId);
    seededUserIds.add(sellerId);

    await prisma.user.create({
        data: {
            id: buyerId,
            stellarAddress: Keypair.random().publicKey(),
        },
    });

    await prisma.user.create({
        data: {
            id: sellerId,
            stellarAddress: Keypair.random().publicKey(),
        },
    });

    return { buyerId, sellerId };
}

async function cleanup() {
    const userIds = Array.from(seededUserIds);
    const escrowIds = Array.from(createdEscrowIds);

    if (escrowIds.length > 0) {
        await prisma.escrow.deleteMany({
            where: { id: { in: escrowIds } },
        });
    }

    if (userIds.length > 0) {
        await prisma.escrow.deleteMany({
            where: {
                OR: [
                    { buyerId: { in: userIds } },
                    { sellerId: { in: userIds } },
                ],
            },
        });
    }

    for (const userId of userIds) {
        try {
            await prisma.user.delete({ where: { id: userId } });
        } catch (error) {
            if (!(error && typeof error === "object" && error.code === "P2025")) {
                throw error;
            }
        }
    }
}

async function run() {
    const { buyerId, sellerId } = await seedUsers();
    console.log(`BUYER_ID=${buyerId}`);
    console.log(`SELLER_ID=${sellerId}`);

    const selfEscrow = await request("POST", "/escrows", {
        buyerId,
        sellerId: buyerId,
        amount: "100",
        assetCode: "USDC",
        expiresAt: futureIso(2 * 60 * 1000),
    });
    assertStatus("self escrow returns 400", selfEscrow.status, 400, selfEscrow.body);

    const create = await request("POST", "/escrows", {
        buyerId,
        sellerId,
        amount: "100",
        assetCode: "USDC",
        expiresAt: futureIso(2 * 60 * 1000),
    });
    assertStatus("create escrow", create.status, 201, create.body);
    const escrowId = create.body?.data?.escrowId;
    if (typeof escrowId === "string" && escrowId.length > 0) {
        createdEscrowIds.add(escrowId);
    }
    const xdr = create.body?.data?.xdr;
    assertCondition("create returns escrowId", typeof escrowId === "string" && escrowId.length > 0, create.body);
    assertCondition("create returns base64 xdr", typeof xdr === "string" && xdr.length > 0, create.body);

    const list = await request("GET", `/escrows?buyerId=${encodeURIComponent(buyerId)}&status=PENDING&page=1&limit=10`);
    assertStatus("list escrows", list.status, 200, list.body);
    const items = list.body?.data?.items || [];
    assertCondition("list includes created escrow", items.some((item) => item.id === escrowId), list.body);

    const get = await request("GET", `/escrows/${escrowId}`);
    assertStatus("get escrow", get.status, 200, get.body);
    assertCondition("get returns pending escrow", get.body?.data?.status === "PENDING", get.body);

    const webhookBody = { escrowId, status: "ACTIVE", stellarTxHash: "abc123hash" };

    if (!WEBHOOK_SECRET) {
        const noConfig = await request("POST", "/escrows/webhook", webhookBody, {
            "x-webhook-secret": "anything",
        });
        assertStatus("webhook without config returns 503", noConfig.status, 503, noConfig.body);
        console.log("\nTimeout test skipped because WEBHOOK_SECRET is not configured (cannot activate escrow via webhook).");
        console.log("\nEscrow flow test passed.");
        return;
    }

    const wrongSecret = await request("POST", "/escrows/webhook", webhookBody, {
        "x-webhook-secret": "wrong-secret",
    });
    assertStatus("webhook wrong secret returns 401", wrongSecret.status, 401, wrongSecret.body);

    const goodSecret = await request("POST", "/escrows/webhook", webhookBody, {
        "x-webhook-secret": WEBHOOK_SECRET,
    });
    assertStatus("webhook valid secret", goodSecret.status, 200, goodSecret.body);
    assertCondition("webhook sets status ACTIVE", goodSecret.body?.data?.status === "ACTIVE", goodSecret.body);

    if (!SKIP_TIMEOUT_TEST) {
        const createShort = await request("POST", "/escrows", {
            buyerId,
            sellerId,
            amount: "50",
            assetCode: "USDC",
            expiresAt: futureIso(10 * 1000),
        });
        assertStatus("create short-lived escrow", createShort.status, 201, createShort.body);
        const shortEscrowId = createShort.body?.data?.escrowId;
        if (typeof shortEscrowId === "string" && shortEscrowId.length > 0) {
            createdEscrowIds.add(shortEscrowId);
        }

        const activateShort = await request(
            "POST",
            "/escrows/webhook",
            { escrowId: shortEscrowId, status: "ACTIVE" },
            { "x-webhook-secret": WEBHOOK_SECRET }
        );
        assertStatus("activate short-lived escrow", activateShort.status, 200, activateShort.body);

        console.log(`Waiting ${TIMEOUT_WAIT_MS}ms for timeout detector (runs every 60000ms)...`);
        await new Promise((resolve) => setTimeout(resolve, TIMEOUT_WAIT_MS));

        const expiredCheck = await request("GET", `/escrows/${shortEscrowId}`);
        assertStatus("fetch short-lived escrow after wait", expiredCheck.status, 200, expiredCheck.body);
        assertCondition(
            "timeout detector marks escrow EXPIRED",
            expiredCheck.body?.data?.status === "EXPIRED",
            expiredCheck.body
        );
    }

    console.log("\nEscrow flow test passed.");
}

run()
    .catch((err) => {
        console.error("\nEscrow flow test failed.");
        console.error(err?.stack || err);
        process.exitCode = 1;
    })
    .finally(async () => {
        try {
            await cleanup();
        } catch (cleanupError) {
            console.error("\nEscrow flow cleanup failed.");
            console.error(cleanupError?.stack || cleanupError);
            process.exitCode = 1;
        } finally {
            await prisma.$disconnect();
        }
    });

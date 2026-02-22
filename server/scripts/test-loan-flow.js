#!/usr/bin/env node

const path = require("path");
const { randomUUID } = require("crypto");
const dotenv = require("dotenv");
const jwt = require("jsonwebtoken");
const { PrismaClient } = require("@prisma/client");
const { Keypair } = require("@stellar/stellar-sdk");

dotenv.config({ path: path.join(__dirname, "..", ".env") });

const API_BASE = process.env.API_BASE || "http://localhost:3001/api";
const JWT_ACCESS_SECRET = process.env.JWT_ACCESS_SECRET || "change-me-in-prod";
const LOAN_CONTRACT_ID = process.env.LOAN_CONTRACT_ID || "";
const prisma = new PrismaClient();

async function request(method, endpoint, token, body) {
    const response = await fetch(`${API_BASE}${endpoint}`, {
        method,
        headers: {
            ...(body ? { "Content-Type": "application/json" } : {}),
            ...(token ? { Authorization: `Bearer ${token}` } : {}),
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

async function seedUser(id) {
    const keypair = Keypair.random();
    const user = await prisma.user.create({
        data: {
            id,
            stellarAddress: keypair.publicKey(),
        },
    });
    return {
        id: user.id,
        stellarAddress: user.stellarAddress,
    };
}

async function run() {
    const borrowerId = randomUUID();
    const lenderId = randomUUID();
    const outsiderId = randomUUID();

    const borrower = await seedUser(borrowerId);
    const lender = await seedUser(lenderId);
    const outsider = await seedUser(outsiderId);
    console.log(`BORROWER_ID=${borrower.id}`);
    console.log(`LENDER_ID=${lender.id}`);

    assertCondition("LOAN_CONTRACT_ID is configured", LOAN_CONTRACT_ID.trim().length > 0, {
        LOAN_CONTRACT_ID,
    });

    const unauthIssueLoan = await request("POST", "/loans", undefined, {
        borrowerId: borrower.id,
        lenderId: lender.id,
        amount: "100",
        collateralAmt: "160",
        assetCode: "USDC",
    });
    assertStatus("unauthenticated issue loan returns 401", unauthIssueLoan.status, 401, unauthIssueLoan.body);

    const token = jwt.sign(
        {
            userId: borrower.id,
            jti: `loan-test-${Date.now()}`,
            walletAddress: borrower.stellarAddress,
        },
        JWT_ACCESS_SECRET,
        { expiresIn: "1h" }
    );
    const outsiderToken = jwt.sign(
        {
            userId: outsider.id,
            jti: `loan-test-outsider-${Date.now()}`,
            walletAddress: outsider.stellarAddress,
        },
        JWT_ACCESS_SECRET,
        { expiresIn: "1h" }
    );

    const selfLending = await request("POST", "/loans", token, {
        borrowerId: borrower.id,
        lenderId: borrower.id,
        amount: "100",
        collateralAmt: "160",
        assetCode: "USDC",
    });
    assertStatus("self-lending returns 400", selfLending.status, 400, selfLending.body);

    const issueLoan = await request("POST", "/loans", token, {
        borrowerId: borrower.id,
        lenderId: lender.id,
        amount: "100",
        collateralAmt: "160",
        assetCode: "USDC",
    });
    assertStatus("issue loan", issueLoan.status, 201, issueLoan.body);
    const loanId = issueLoan.body?.data?.loanId;
    const xdr = issueLoan.body?.data?.xdr;
    assertCondition("issue loan returns loanId", Boolean(loanId), issueLoan.body);
    assertCondition("issue loan returns unsigned xdr", typeof xdr === "string" && xdr.length > 0, issueLoan.body);

    const outsiderIssueLoan = await request("POST", "/loans", outsiderToken, {
        borrowerId: borrower.id,
        lenderId: lender.id,
        amount: "100",
        collateralAmt: "160",
        assetCode: "USDC",
    });
    assertStatus("outsider issue loan returns 403", outsiderIssueLoan.status, 403, outsiderIssueLoan.body);

    const getLoan = await request("GET", `/loans/${loanId}`, token);
    assertStatus("get loan", getLoan.status, 200, getLoan.body);
    assertCondition("new loan starts as PENDING", getLoan.body?.data?.status === "PENDING", getLoan.body);

    const listPending = await request("GET", "/loans?status=PENDING", token);
    assertStatus("list pending loans", listPending.status, 200, listPending.body);
    const pendingIds = (listPending.body?.data || []).map((loan) => loan.id);
    assertCondition("pending list includes created loan", pendingIds.includes(loanId), listPending.body);

    const repayPartial = await request("POST", "/loans/repay", token, {
        loanId,
        amount: "40",
    });
    assertStatus("record partial repayment", repayPartial.status, 200, repayPartial.body);
    assertCondition("partial repayment not fully repaid", repayPartial.body?.data?.fullyRepaid === false, repayPartial.body);
    assertCondition("partial repayment sets ACTIVE", repayPartial.body?.data?.loan?.status === "ACTIVE", repayPartial.body);
    assertCondition("partial repayment outstanding values are strings", typeof repayPartial.body?.data?.outstandingBefore === "string" && typeof repayPartial.body?.data?.outstandingAfter === "string", repayPartial.body);

    const outsiderRepay = await request("POST", "/loans/repay", outsiderToken, {
        loanId,
        amount: "10",
    });
    assertStatus("outsider repayment returns 403", outsiderRepay.status, 403, outsiderRepay.body);

    const overpay = await request("POST", "/loans/repay", token, {
        loanId,
        amount: "70",
    });
    assertStatus("overpayment returns 400", overpay.status, 400, overpay.body);

    const repayFinal = await request("POST", "/loans/repay", token, {
        loanId,
        amount: "60",
    });
    assertStatus("record final repayment", repayFinal.status, 200, repayFinal.body);
    assertCondition("final repayment fully repaid", repayFinal.body?.data?.fullyRepaid === true, repayFinal.body);
    assertCondition("final repayment sets REPAID", repayFinal.body?.data?.loan?.status === "REPAID", repayFinal.body);

    const listRepaid = await request("GET", "/loans?status=REPAID", token);
    assertStatus("list repaid loans", listRepaid.status, 200, listRepaid.body);
    const repaidIds = (listRepaid.body?.data || []).map((loan) => loan.id);
    assertCondition("repaid list includes created loan", repaidIds.includes(loanId), listRepaid.body);

    const invalidStatus = await request("GET", "/loans?status=INVALID", token);
    assertStatus("invalid status filter returns 400", invalidStatus.status, 400, invalidStatus.body);

    const badCollateral = await request("POST", "/loans", token, {
        borrowerId: borrower.id,
        lenderId: lender.id,
        amount: "100",
        collateralAmt: "120",
        assetCode: "USDC",
    });
    assertStatus("under-collateralized loan returns 400", badCollateral.status, 400, badCollateral.body);

    console.log("\nLoan flow test passed.");
}

run()
    .catch((err) => {
        console.error("\nLoan flow test failed.");
        console.error(err?.stack || err);
        process.exitCode = 1;
    })
    .finally(async () => {
        await prisma.$disconnect();
    });

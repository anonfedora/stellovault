/**
 * StelloVault — Prisma Seed Script
 *
 * Populates the database with realistic-looking local development data.
 * Run with: npx prisma db seed
 */

import { PrismaClient, EscrowStatus, LoanStatus, CollateralStatus, GovernanceStatus } from '@prisma/client';

const prisma = new PrismaClient();

async function main() {
    console.log('🌱  Seeding database...');

    // ── Users ──────────────────────────────────────────────────────────────────
    const alice = await prisma.user.upsert({
        where: { stellarAddress: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN' },
        update: {},
        create: {
            stellarAddress: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
            name: 'Alice (Dev)',
            role: 'USER',
        },
    });

    const bob = await prisma.user.upsert({
        where: { stellarAddress: 'GCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGZQ3S1RRRMF3SKBPZWRQJ' },
        update: {},
        create: {
            stellarAddress: 'GCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGZQ3S1RRRMF3SKBPZWRQJ',
            name: 'Bob (Dev)',
            role: 'MERCHANT',
        },
    });

    console.log(`  ✔  Users: alice=${alice.id}  bob=${bob.id}`);

    // ── Wallets ────────────────────────────────────────────────────────────────
    const aliceWallet = await prisma.wallet.upsert({
        where: { address: alice.stellarAddress },
        update: {},
        create: {
            userId: alice.id,
            address: alice.stellarAddress,
            isPrimary: true,
            label: 'Primary',
            verifiedAt: new Date(),
        },
    });

    const bobWallet = await prisma.wallet.upsert({
        where: { address: bob.stellarAddress },
        update: {},
        create: {
            userId: bob.id,
            address: bob.stellarAddress,
            isPrimary: true,
            label: 'Primary',
            verifiedAt: new Date(),
        },
    });

    console.log(`  ✔  Wallets: aliceWallet=${aliceWallet.id}  bobWallet=${bobWallet.id}`);

    // ── Escrow ─────────────────────────────────────────────────────────────────
    const escrow = await prisma.escrow.create({
        data: {
            buyerId: alice.id,
            sellerId: bob.id,
            amount: 500.0,
            assetCode: 'USDC',
            status: EscrowStatus.ACTIVE,
            stellarTxHash: 'abc123deadbeef',
            expiresAt: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000), // +7 days
        },
    });

    console.log(`  ✔  Escrow: ${escrow.id}`);

    // ── Collateral ─────────────────────────────────────────────────────────────
    const collateral = await prisma.collateral.create({
        data: {
            escrowId: escrow.id,
            assetCode: 'XLM',
            amount: 1000.0,
            metadataHash: 'sha256:abc123',
            status: CollateralStatus.LOCKED,
        },
    });

    console.log(`  ✔  Collateral: ${collateral.id}`);

    // ── Loan & Repayment ───────────────────────────────────────────────────────
    const loan = await prisma.loan.create({
        data: {
            borrowerId: alice.id,
            lenderId: bob.id,
            amount: 300.0,
            assetCode: 'USDC',
            interestRate: 0.05,
            status: LoanStatus.ACTIVE,
            dueDate: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000), // +30 days
            collateralId: collateral.id,
        },
    });

    await prisma.repayment.create({
        data: {
            loanId: loan.id,
            amount: 50.0,
            paidAt: new Date(),
        },
    });

    console.log(`  ✔  Loan: ${loan.id}  (with 1 repayment)`);

    // ── Oracle Event ───────────────────────────────────────────────────────────
    const oracleEvent = await prisma.oracleEvent.create({
        data: {
            escrowId: escrow.id,
            oracleAddress: 'GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37',
            confirmationType: 'ESCROW_ACTIVE',
            signature: 'sig:deadbeef1234',
            confirmedAt: new Date(),
        },
    });

    console.log(`  ✔  OracleEvent: ${oracleEvent.id}`);

    // ── Governance ─────────────────────────────────────────────────────────────
    const proposal = await prisma.governanceProposal.create({
        data: {
            title: 'Increase collateral ratio to 150%',
            description: 'Proposal to improve protocol safety by raising the minimum collateral ratio from 120% to 150%.',
            proposerId: alice.id,
            status: GovernanceStatus.OPEN,
            endsAt: new Date(Date.now() + 14 * 24 * 60 * 60 * 1000), // +14 days
        },
    });

    await prisma.governanceVote.create({
        data: {
            proposalId: proposal.id,
            voterAddress: bob.stellarAddress,
            vote: 'YES',
            weight: 100.0,
        },
    });

    console.log(`  ✔  GovernanceProposal: ${proposal.id}  (with 1 vote)`);

    // ── Risk Score ─────────────────────────────────────────────────────────────
    const riskScore = await prisma.riskScore.create({
        data: {
            walletAddress: alice.stellarAddress,
            score: 720,
            components: {
                transactionHistory: 200,
                repaymentRecord: 250,
                collateralCoverage: 180,
                disputeHistory: 90,
            },
            recordedAt: new Date(),
        },
    });

    console.log(`  ✔  RiskScore: ${riskScore.id}  (score=${riskScore.score})`);

    console.log('\n✅  Seed complete.');
}

main()
    .catch((e) => {
        console.error('❌  Seed failed:', e);
        process.exit(1);
    })
    .finally(async () => {
        await prisma.$disconnect();
    });

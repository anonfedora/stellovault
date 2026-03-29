import { prisma } from "../services/database.service";

type PlatformStats = {
    totalEscrows: number;
    fundedEscrows: number;
    releasedEscrows: number;
    disputedEscrows: number;
    totalLoans: number;
    activeLoans: number;
    defaultedLoans: number;
    totalVolumeUSDC: string;
    totalUsers: number;
    activeWallets: number;
    governanceProposals: number;
    participationRate: number;
};

type ProtocolAnalytics = {
    tvl: string;
    totalVolume: string;
    avgInterestRate: string;
    defaultRate: number;
};

class AnalyticsService {
    private cache: { ts: number; data: PlatformStats } | null = null;
    private ttl = 60 * 1000; // 60 seconds
    private protocolCache: { ts: number; data: ProtocolAnalytics } | null = null;
    private protocolTtl = 24 * 60 * 60 * 1000; // 24 hours

    async getPlatformStats(): Promise<PlatformStats> {
        const now = Date.now();
        if (this.cache && now - this.cache.ts < this.ttl) {
            return this.cache.data;
        }

        try {
            const [
                totalEscrows,
                fundedEscrows,
                releasedEscrows,
                disputedEscrows,
                totalLoans,
                activeLoans,
                defaultedLoans,
                escrowSumRes,
                loanSumRes,
                totalUsers,
                activeWallets,
            ] = await Promise.all([
                prisma.escrow.count(),
                prisma.escrow.count({ where: { status: "FUNDED" } }),
                prisma.escrow.count({ where: { status: "RELEASED" } }),
                prisma.escrow.count({ where: { status: "DISPUTED" } }),
                prisma.loan.count(),
                prisma.loan.count({ where: { status: "ACTIVE" } }),
                prisma.loan.count({ where: { status: "DEFAULTED" } }),
                prisma.escrow.aggregate({ _sum: { amount: true }, where: { assetCode: "USDC" } }),
                prisma.loan.aggregate({ _sum: { amount: true }, where: { assetCode: "USDC" } }),
                prisma.user.count(),
                prisma.wallet.count({ where: { verifiedAt: { not: null } } }),
            ]);

            const escrowSum = (escrowSumRes as any)?._sum?.amount ?? 0;
            const loanSum = (loanSumRes as any)?._sum?.amount ?? 0;
            const totalVolume = (typeof escrowSum === "string" || typeof escrowSum === "number")
                ? Number(escrowSum) + Number(loanSum)
                : (escrowSum as any).toNumber() + (loanSum as any).toNumber();

            // governance tables may not exist yet; attempt safe queries
            let governanceProposals = 0;
            let participationRate = 0;
            try {
                // @ts-ignore - model may not be present in schema
                governanceProposals = await (prisma as any).governanceProposal?.count?.() ?? 0;
                // participationRate calculation depends on votes table; default to 0 when not available
                participationRate = 0;
            } catch (_) {
                governanceProposals = 0;
                participationRate = 0;
            }

            const data: PlatformStats = {
                totalEscrows,
                fundedEscrows,
                releasedEscrows,
                disputedEscrows,
                totalLoans,
                activeLoans,
                defaultedLoans,
                totalVolumeUSDC: totalVolume.toString(),
                totalUsers,
                activeWallets,
                governanceProposals,
                participationRate,
            };

            this.cache = { ts: now, data };
            return data;
        } catch (err) {
            // In case of unexpected errors, rethrow to be handled by controller
            throw err;
        }
    }

    async getProtocolAnalytics(): Promise<ProtocolAnalytics> {
        const now = Date.now();
        if (this.protocolCache && now - this.protocolCache.ts < this.protocolTtl) {
            return this.protocolCache.data;
        }

        const [tvlRes, issuedCount, defaultedCount, loanAggRows] = await Promise.all([
            prisma.escrow.aggregate({
                _sum: { amount: true },
                where: { status: { in: ["FUNDED", "DISPUTED"] } },
            }),
            prisma.loan.count({ where: { NOT: { status: "PENDING" } } }),
            prisma.loan.count({ where: { status: "DEFAULTED" } }),
            prisma.$queryRaw<
                Array<{
                    sum_amount: any;
                    sum_amount_times_rate: any;
                }>
            >`
                SELECT
                    COALESCE(SUM("amount"), 0) AS sum_amount,
                    COALESCE(SUM("amount" * "interestRate"), 0) AS sum_amount_times_rate
                FROM "Loan"
                WHERE "status" <> 'PENDING'
            `,
        ]);

        const tvlSum = (tvlRes as any)?._sum?.amount ?? 0;
        const sumAmount = loanAggRows?.[0]?.sum_amount ?? 0;
        const sumAmountTimesRate = loanAggRows?.[0]?.sum_amount_times_rate ?? 0;

        const toNumber = (v: any): number => {
            if (v == null) return 0;
            if (typeof v === "number") return v;
            if (typeof v === "bigint") return Number(v);
            if (typeof v === "string") return Number(v);
            if (typeof v?.toNumber === "function") return v.toNumber();
            return Number(v);
        };

        const tvlNum = toNumber(tvlSum);
        const volumeNum = toNumber(sumAmount);
        const weightedRateNum =
            volumeNum > 0 ? toNumber(sumAmountTimesRate) / volumeNum : 0;

        const defaultRate = issuedCount > 0 ? defaultedCount / issuedCount : 0;

        const data: ProtocolAnalytics = {
            tvl: tvlNum.toString(),
            totalVolume: volumeNum.toString(),
            avgInterestRate: weightedRateNum.toString(),
            defaultRate,
        };

        this.protocolCache = { ts: now, data };
        return data;
    }
}

export default new AnalyticsService();

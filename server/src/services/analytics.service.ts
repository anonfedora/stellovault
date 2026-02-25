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

class AnalyticsService {
    private cache: { ts: number; data: PlatformStats } | null = null;
    private ttl = 60 * 1000; // 60 seconds

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
}

export default new AnalyticsService();

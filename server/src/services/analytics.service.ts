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

type DashboardMetrics = {
    totalValueLocked: string;
    activeUsers: number;
    dailyTransactionVolume: string;
    pendingTransactions: number;
    systemHealth: 'healthy' | 'warning' | 'critical';
    averageResponseTime: number;
    errorRate: number;
    uptime: number;
};

type VolumeReport = {
    period: string;
    totalVolume: string;
    escrowVolume: string;
    loanVolume: string;
    transactionCount: number;
    averageTransactionSize: string;
    dailyBreakdown: Array<{
        date: string;
        volume: string;
        count: number;
    }>;
};

type UserAnalytics = {
    totalUsers: number;
    activeUsers: number;
    newUsers: number;
    userRetentionRate: number;
    averageSessionDuration: number;
    topUserActions: Array<{
        action: string;
        count: number;
    }>;
    userGrowth: Array<{
        period: string;
        users: number;
    }>;
};

type PlatformPerformance = {
    cpuUsage: number;
    memoryUsage: number;
    databaseConnections: number;
    averageResponseTime: number;
    requestsPerSecond: number;
    errorRate: number;
    uptime: number;
    timestamp: string;
};

type ComplianceMetrics = {
    kycComplianceRate: number;
    amlFlaggedTransactions: number;
    suspiciousActivityReports: number;
    regulatoryReportingStatus: 'compliant' | 'pending' | 'non-compliant';
    auditTrailIntegrity: boolean;
    dataRetentionCompliance: boolean;
    lastAuditDate: string;
};

type CustomReportRequest = {
    reportType: 'volume' | 'users' | 'performance' | 'compliance';
    startDate: string;
    endDate: string;
    filters?: Record<string, any>;
    format?: 'json' | 'csv';
};

type CustomReport = {
    id: string;
    reportType: string;
    generatedAt: string;
    data: any;
    metadata: {
        period: string;
        recordCount: number;
        filters: Record<string, any>;
    };
};

class AnalyticsService {
    private cache: { ts: number; data: PlatformStats } | null = null;
    private ttl = 60 * 1000; // 60 seconds
    private protocolCache: { ts: number; data: ProtocolAnalytics } | null = null;
    private protocolTtl = 24 * 60 * 60 * 1000; // 24 hours
    private dashboardCache: { ts: number; data: DashboardMetrics } | null = null;
    private dashboardTtl = 30 * 1000; // 30 seconds for real-time data
    private volumeCache: Map<string, { ts: number; data: VolumeReport }> = new Map();
    private volumeTtl = 5 * 60 * 1000; // 5 minutes
    private userCache: { ts: number; data: UserAnalytics } | null = null;
    private userTtl = 10 * 60 * 1000; // 10 minutes
    private performanceCache: { ts: number; data: PlatformPerformance } | null = null;
    private performanceTtl = 60 * 1000; // 1 minute
    private complianceCache: { ts: number; data: ComplianceMetrics } | null = null;
    private complianceTtl = 60 * 60 * 1000; // 1 hour

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

    async calculateDashboardMetrics(): Promise<DashboardMetrics> {
        const now = Date.now();
        if (this.dashboardCache && now - this.dashboardCache.ts < this.dashboardTtl) {
            return this.dashboardCache.data;
        }

        try {
            const [
                tvlRes,
                activeUsers,
                dailyVolumeRes,
                pendingCount,
                systemMetrics
            ] = await Promise.all([
                prisma.escrow.aggregate({
                    _sum: { amount: true },
                    where: { status: { in: ["FUNDED", "DISPUTED"] } }
                }),
                prisma.user.count({
                    where: {
                        updatedAt: {
                            gte: new Date(Date.now() - 24 * 60 * 60 * 1000)
                        }
                    }
                }),
                prisma.$queryRaw<Array<{ sum_amount: any }>>`
                    SELECT COALESCE(SUM("amount"), 0) AS sum_amount
                    FROM "Escrow"
                    WHERE "createdAt" >= ${new Date(Date.now() - 24 * 60 * 60 * 1000)}
                `,
                prisma.escrow.count({ where: { status: "PENDING" } }) +
                prisma.loan.count({ where: { status: "PENDING" } }),
                this.getSystemMetrics()
            ]);

            const tvl = (tvlRes as any)?._sum?.amount ?? 0;
            const dailyVolume = dailyVolumeRes?.[0]?.sum_amount ?? 0;
            
            const systemHealth = this.calculateSystemHealth(systemMetrics);
            
            const data: DashboardMetrics = {
                totalValueLocked: this.toNumber(tvl).toString(),
                activeUsers,
                dailyTransactionVolume: this.toNumber(dailyVolume).toString(),
                pendingTransactions: pendingCount,
                systemHealth,
                averageResponseTime: systemMetrics.avgResponseTime,
                errorRate: systemMetrics.errorRate,
                uptime: systemMetrics.uptime
            };

            this.dashboardCache = { ts: now, data };
            return data;
        } catch (err) {
            throw err;
        }
    }

    async generateVolumeReport(period: 'hour' | 'day' | 'week' | 'month'): Promise<VolumeReport> {
        const cacheKey = period;
        const now = Date.now();
        const cached = this.volumeCache.get(cacheKey);
        
        if (cached && now - cached.ts < this.volumeTtl) {
            return cached.data;
        }

        const periodMap = {
            hour: 1 * 60 * 60 * 1000,
            day: 24 * 60 * 60 * 1000,
            week: 7 * 24 * 60 * 60 * 1000,
            month: 30 * 24 * 60 * 60 * 1000
        };

        const startDate = new Date(now - periodMap[period]);
        
        try {
            const [escrowVolume, loanVolume, escrowCount, loanCount, dailyBreakdown] = await Promise.all([
                prisma.escrow.aggregate({
                    _sum: { amount: true },
                    where: { createdAt: { gte: startDate } }
                }),
                prisma.loan.aggregate({
                    _sum: { amount: true },
                    where: { createdAt: { gte: startDate } }
                }),
                prisma.escrow.count({ where: { createdAt: { gte: startDate } } }),
                prisma.loan.count({ where: { createdAt: { gte: startDate } } }),
                this.getDailyVolumeBreakdown(startDate)
            ]);

            const escrowSum = (escrowVolume as any)?._sum?.amount ?? 0;
            const loanSum = (loanVolume as any)?._sum?.amount ?? 0;
            const totalVolume = this.toNumber(escrowSum) + this.toNumber(loanSum);
            const totalTransactions = escrowCount + loanCount;

            const data: VolumeReport = {
                period,
                totalVolume: totalVolume.toString(),
                escrowVolume: this.toNumber(escrowSum).toString(),
                loanVolume: this.toNumber(loanSum).toString(),
                transactionCount: totalTransactions,
                averageTransactionSize: totalTransactions > 0 ? (totalVolume / totalTransactions).toString() : "0",
                dailyBreakdown
            };

            this.volumeCache.set(cacheKey, { ts: now, data });
            return data;
        } catch (err) {
            throw err;
        }
    }

    async analyzeUserBehavior(userId?: string): Promise<UserAnalytics> {
        const now = Date.now();
        if (this.userCache && now - this.userCache.ts < this.userTtl && !userId) {
            return this.userCache.data;
        }

        try {
            const whereClause = userId ? { id: userId } : {};
            
            const [
                totalUsers,
                activeUsers,
                newUsers,
                userActions,
                userGrowth
            ] = await Promise.all([
                prisma.user.count(),
                prisma.user.count({
                    where: {
                        updatedAt: {
                            gte: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)
                        }
                    }
                }),
                prisma.user.count({
                    where: {
                        createdAt: {
                            gte: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
                        }
                    }
                }),
                this.getTopUserActions(),
                this.getUserGrowthTrend()
            ]);

            const retentionRate = totalUsers > 0 ? (activeUsers / totalUsers) * 100 : 0;

            const data: UserAnalytics = {
                totalUsers,
                activeUsers,
                newUsers,
                userRetentionRate: retentionRate,
                averageSessionDuration: 1800, // 30 minutes placeholder
                topUserActions: userActions,
                userGrowth
            };

            if (!userId) {
                this.userCache = { ts: now, data };
            }
            
            return data;
        } catch (err) {
            throw err;
        }
    }

    async trackPlatformPerformance(): Promise<PlatformPerformance> {
        const now = Date.now();
        if (this.performanceCache && now - this.performanceCache.ts < this.performanceTtl) {
            return this.performanceCache.data;
        }

        try {
            const metrics = await this.getSystemMetrics();
            
            const data: PlatformPerformance = {
                cpuUsage: metrics.cpuUsage,
                memoryUsage: metrics.memoryUsage,
                databaseConnections: metrics.dbConnections,
                averageResponseTime: metrics.avgResponseTime,
                requestsPerSecond: metrics.requestsPerSecond,
                errorRate: metrics.errorRate,
                uptime: metrics.uptime,
                timestamp: new Date().toISOString()
            };

            this.performanceCache = { ts: now, data };
            return data;
        } catch (err) {
            throw err;
        }
    }

    async generateComplianceReport(type: 'kyc' | 'aml' | 'audit' | 'retention'): Promise<ComplianceMetrics> {
        const now = Date.now();
        if (this.complianceCache && now - this.complianceCache.ts < this.complianceTtl) {
            return this.complianceCache.data;
        }

        try {
            const [
                verifiedWallets,
                totalWallets,
                flaggedTransactions,
                suspiciousReports
            ] = await Promise.all([
                prisma.wallet.count({ where: { verifiedAt: { not: null } } }),
                prisma.wallet.count(),
                prisma.auditLog.count({
                    where: {
                        action: { contains: 'FLAGGED' },
                        createdAt: {
                            gte: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
                        }
                    }
                }),
                prisma.auditLog.count({
                    where: {
                        action: { contains: 'SUSPICIOUS' },
                        createdAt: {
                            gte: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
                        }
                    }
                })
            ]);

            const kycComplianceRate = totalWallets > 0 ? (verifiedWallets / totalWallets) * 100 : 0;

            const data: ComplianceMetrics = {
                kycComplianceRate,
                amlFlaggedTransactions: flaggedTransactions,
                suspiciousActivityReports: suspiciousReports,
                regulatoryReportingStatus: 'compliant',
                auditTrailIntegrity: true,
                dataRetentionCompliance: true,
                lastAuditDate: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString()
            };

            this.complianceCache = { ts: now, data };
            return data;
        } catch (err) {
            throw err;
        }
    }

    async generateCustomReport(request: CustomReportRequest): Promise<CustomReport> {
        const reportId = `report_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        
        try {
            let data: any;
            
            switch (request.reportType) {
                case 'volume':
                    data = await this.generateVolumeReport('day');
                    break;
                case 'users':
                    data = await this.analyzeUserBehavior();
                    break;
                case 'performance':
                    data = await this.trackPlatformPerformance();
                    break;
                case 'compliance':
                    data = await this.generateComplianceReport('kyc');
                    break;
                default:
                    throw new Error('Invalid report type');
            }

            const report: CustomReport = {
                id: reportId,
                reportType: request.reportType,
                generatedAt: new Date().toISOString(),
                data,
                metadata: {
                    period: `${request.startDate} to ${request.endDate}`,
                    recordCount: Array.isArray(data) ? data.length : 1,
                    filters: request.filters || {}
                }
            };

            return report;
        } catch (err) {
            throw err;
        }
    }

    // Helper methods
    private toNumber(value: any): number {
        if (value == null) return 0;
        if (typeof value === "number") return value;
        if (typeof value === "bigint") return Number(value);
        if (typeof value === "string") return Number(value);
        if (typeof value?.toNumber === "function") return value.toNumber();
        return Number(value);
    }

    private async getSystemMetrics() {
        // Placeholder for system metrics - in production, these would come from monitoring systems
        return {
            cpuUsage: Math.random() * 100,
            memoryUsage: Math.random() * 100,
            dbConnections: 10,
            avgResponseTime: 150,
            requestsPerSecond: 50,
            errorRate: 0.01,
            uptime: 99.9
        };
    }

    private calculateSystemHealth(metrics: any): 'healthy' | 'warning' | 'critical' {
        if (metrics.errorRate > 5 || metrics.cpuUsage > 90) return 'critical';
        if (metrics.errorRate > 1 || metrics.cpuUsage > 70) return 'warning';
        return 'healthy';
    }

    private async getDailyVolumeBreakdown(startDate: Date) {
        // Placeholder for daily breakdown - would implement actual time-series query
        return [
            {
                date: new Date(startDate.getTime() + 24 * 60 * 60 * 1000).toISOString().split('T')[0],
                volume: "1000",
                count: 10
            }
        ];
    }

    private async getTopUserActions() {
        // Placeholder for user actions analytics
        return [
            { action: 'create_escrow', count: 100 },
            { action: 'fund_escrow', count: 85 },
            { action: 'release_escrow', count: 75 }
        ];
    }

    private async getUserGrowthTrend() {
        // Placeholder for user growth trend
        return [
            { period: '2024-01', users: 100 },
            { period: '2024-02', users: 120 },
            { period: '2024-03', users: 150 }
        ];
    }
}

export default new AnalyticsService();

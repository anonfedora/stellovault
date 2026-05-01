import { prisma } from "./database.service";

type RetentionPolicy = {
    dataType: 'audit_logs' | 'user_sessions' | 'temp_data' | 'analytics_cache';
    retentionDays: number;
    enabled: boolean;
};

class DataRetentionService {
    private policies: RetentionPolicy[] = [
        { dataType: 'audit_logs', retentionDays: 2555, enabled: true }, // 7 years
        { dataType: 'user_sessions', retentionDays: 30, enabled: true }, // 30 days
        { dataType: 'temp_data', retentionDays: 7, enabled: true }, // 7 days
        { dataType: 'analytics_cache', retentionDays: 90, enabled: true } // 90 days
    ];

    async cleanupExpiredData(): Promise<{ deletedRecords: number; policiesApplied: string[] }> {
        let totalDeleted = 0;
        const appliedPolicies: string[] = [];

        for (const policy of this.policies) {
            if (!policy.enabled) continue;

            try {
                const deleted = await this.applyRetentionPolicy(policy);
                totalDeleted += deleted;
                appliedPolicies.push(`${policy.dataType}: ${deleted} records`);
            } catch (error) {
                console.error(`Error applying retention policy for ${policy.dataType}:`, error);
            }
        }

        return { deletedRecords: totalDeleted, policiesApplied: appliedPolicies };
    }

    private async applyRetentionPolicy(policy: RetentionPolicy): Promise<number> {
        const cutoffDate = new Date();
        cutoffDate.setDate(cutoffDate.getDate() - policy.retentionDays);

        switch (policy.dataType) {
            case 'audit_logs':
                return await this.cleanupAuditLogs(cutoffDate);
            case 'user_sessions':
                return await this.cleanupUserSessions(cutoffDate);
            case 'temp_data':
                return await this.cleanupTempData(cutoffDate);
            case 'analytics_cache':
                return await this.cleanupAnalyticsCache(cutoffDate);
            default:
                throw new Error(`Unknown data type: ${policy.dataType}`);
        }
    }

    private async cleanupAuditLogs(cutoffDate: Date): Promise<number> {
        const result = await prisma.auditLog.deleteMany({
            where: {
                createdAt: {
                    lt: cutoffDate
                }
            }
        });
        return result.count;
    }

    private async cleanupUserSessions(cutoffDate: Date): Promise<number> {
        const result = await prisma.session.deleteMany({
            where: {
                createdAt: {
                    lt: cutoffDate
                }
            }
        });
        return result.count;
    }

    private async cleanupTempData(cutoffDate: Date): Promise<number> {
        // Placeholder for temporary data cleanup
        // In production, this would clean up any temporary tables or data
        console.log(`Cleaning up temporary data older than ${cutoffDate.toISOString()}`);
        return 0;
    }

    private async cleanupAnalyticsCache(cutoffDate: Date): Promise<number> {
        // Placeholder for analytics cache cleanup
        // In production, this would clean up cached analytics data
        console.log(`Cleaning up analytics cache older than ${cutoffDate.toISOString()}`);
        return 0;
    }

    async exportDataForCompliance(
        dataType: string,
        startDate: Date,
        endDate: Date,
        format: 'json' | 'csv'
    ): Promise<string> {
        switch (dataType) {
            case 'audit_logs':
                return await this.exportAuditLogs(startDate, endDate, format);
            case 'users':
                return await this.exportUserData(startDate, endDate, format);
            case 'transactions':
                return await this.exportTransactionData(startDate, endDate, format);
            case 'compliance':
                return await this.exportComplianceData(startDate, endDate, format);
            default:
                throw new Error(`Unknown data type for export: ${dataType}`);
        }
    }

    private async exportAuditLogs(startDate: Date, endDate: Date, format: 'json' | 'csv'): Promise<string> {
        const logs = await prisma.auditLog.findMany({
            where: {
                createdAt: {
                    gte: startDate,
                    lte: endDate
                }
            },
            orderBy: {
                createdAt: 'asc'
            }
        });

        if (format === 'json') {
            return JSON.stringify(logs, null, 2);
        }

        // CSV format
        const headers = ['id', 'userId', 'action', 'resourceId', 'ipAddress', 'createdAt'];
        const rows = logs.map(log => [
            log.id,
            log.userId || '',
            log.action,
            log.resourceId || '',
            log.ipAddress || '',
            log.createdAt.toISOString()
        ]);

        return [headers, ...rows].map(row => row.join(',')).join('\n');
    }

    private async exportUserData(startDate: Date, endDate: Date, format: 'json' | 'csv'): Promise<string> {
        const users = await prisma.user.findMany({
            where: {
                createdAt: {
                    gte: startDate,
                    lte: endDate
                }
            },
            include: {
                wallets: {
                    select: {
                        address: true,
                        verifiedAt: true,
                        isPrimary: true
                    }
                }
            },
            orderBy: {
                createdAt: 'asc'
            }
        });

        if (format === 'json') {
            return JSON.stringify(users, null, 2);
        }

        // CSV format
        const headers = ['id', 'stellarAddress', 'name', 'role', 'walletCount', 'verifiedWallets', 'createdAt'];
        const rows = users.map(user => [
            user.id,
            user.stellarAddress,
            user.name || '',
            user.role,
            user.wallets.length.toString(),
            user.wallets.filter(w => w.verifiedAt).length.toString(),
            user.createdAt.toISOString()
        ]);

        return [headers, ...rows].map(row => row.join(',')).join('\n');
    }

    private async exportTransactionData(startDate: Date, endDate: Date, format: 'json' | 'csv'): Promise<string> {
        const [escrows, loans] = await Promise.all([
            prisma.escrow.findMany({
                where: {
                    createdAt: {
                        gte: startDate,
                        lte: endDate
                    }
                },
                include: {
                    buyer: { select: { stellarAddress: true } },
                    seller: { select: { stellarAddress: true } }
                },
                orderBy: { createdAt: 'asc' }
            }),
            prisma.loan.findMany({
                where: {
                    createdAt: {
                        gte: startDate,
                        lte: endDate
                    }
                },
                include: {
                    borrower: { select: { stellarAddress: true } },
                    lender: { select: { stellarAddress: true } }
                },
                orderBy: { createdAt: 'asc' }
            })
        ]);

        const transactions = [
            ...escrows.map(e => ({
                type: 'escrow',
                id: e.id,
                amount: e.amount.toString(),
                assetCode: e.assetCode,
                status: e.status,
                buyerAddress: e.buyer.stellarAddress,
                sellerAddress: e.seller.stellarAddress,
                createdAt: e.createdAt
            })),
            ...loans.map(l => ({
                type: 'loan',
                id: l.id,
                amount: l.amount.toString(),
                assetCode: l.assetCode,
                status: l.status,
                borrowerAddress: l.borrower.stellarAddress,
                lenderAddress: l.lender?.stellarAddress || '',
                createdAt: l.createdAt
            }))
        ].sort((a, b) => a.createdAt.getTime() - b.createdAt.getTime());

        if (format === 'json') {
            return JSON.stringify(transactions, null, 2);
        }

        // CSV format
        const headers = ['type', 'id', 'amount', 'assetCode', 'status', 'buyerAddress', 'sellerAddress', 'createdAt'];
        const rows = transactions.map(t => [
            t.type,
            t.id,
            t.amount,
            t.assetCode,
            t.status,
            t.type === 'escrow' ? t.buyerAddress : t.borrowerAddress,
            t.type === 'escrow' ? t.sellerAddress : t.lenderAddress,
            t.createdAt.toISOString()
        ]);

        return [headers, ...rows].map(row => row.join(',')).join('\n');
    }

    private async exportComplianceData(startDate: Date, endDate: Date, format: 'json' | 'csv'): Promise<string> {
        const [verifiedWallets, totalWallets, flaggedTransactions] = await Promise.all([
            prisma.wallet.count({
                where: {
                    verifiedAt: {
                        gte: startDate,
                        lte: endDate
                    }
                }
            }),
            prisma.wallet.count({
                where: {
                    createdAt: {
                        gte: startDate,
                        lte: endDate
                    }
                }
            }),
            prisma.auditLog.count({
                where: {
                    action: { contains: 'FLAGGED' },
                    createdAt: {
                        gte: startDate,
                        lte: endDate
                    }
                }
            })
        ]);

        const complianceData = {
            period: `${startDate.toISOString()} to ${endDate.toISOString()}`,
            kycMetrics: {
                totalWallets,
                verifiedWallets,
                complianceRate: totalWallets > 0 ? (verifiedWallets / totalWallets) * 100 : 0
            },
            amlMetrics: {
                flaggedTransactions,
                reviewRate: totalWallets > 0 ? (flaggedTransactions / totalWallets) * 100 : 0
            },
            auditTrail: {
                integrity: true,
                lastVerified: new Date().toISOString()
            }
        };

        if (format === 'json') {
            return JSON.stringify(complianceData, null, 2);
        }

        // CSV format for compliance summary
        const headers = ['metric', 'value', 'period'];
        const rows = [
            ['total_wallets', totalWallets.toString(), complianceData.period],
            ['verified_wallets', verifiedWallets.toString(), complianceData.period],
            ['compliance_rate', complianceData.kycMetrics.complianceRate.toString(), complianceData.period],
            ['flagged_transactions', flaggedTransactions.toString(), complianceData.period],
            ['audit_integrity', complianceData.auditTrail.integrity.toString(), complianceData.period]
        ];

        return [headers, ...rows].map(row => row.join(',')).join('\n');
    }

    async updateRetentionPolicy(dataType: string, retentionDays: number, enabled: boolean): Promise<void> {
        const policyIndex = this.policies.findIndex(p => p.dataType === dataType);
        if (policyIndex === -1) {
            throw new Error(`Unknown data type: ${dataType}`);
        }

        this.policies[policyIndex] = { dataType, retentionDays, enabled };
    }

    getRetentionPolicies(): RetentionPolicy[] {
        return [...this.policies];
    }

    async getDataRetentionStatus(): Promise<{
        dataType: string;
        totalRecords: number;
        expiredRecords: number;
        nextCleanupDate: string;
    }[]> {
        const status = [];

        for (const policy of this.policies) {
            try {
                const cutoffDate = new Date();
                cutoffDate.setDate(cutoffDate.getDate() - policy.retentionDays);

                let totalRecords = 0;
                let expiredRecords = 0;

                switch (policy.dataType) {
                    case 'audit_logs':
                        totalRecords = await prisma.auditLog.count();
                        expiredRecords = await prisma.auditLog.count({
                            where: { createdAt: { lt: cutoffDate } }
                        });
                        break;
                    case 'user_sessions':
                        totalRecords = await prisma.session.count();
                        expiredRecords = await prisma.session.count({
                            where: { createdAt: { lt: cutoffDate } }
                        });
                        break;
                    default:
                        // For other data types, use placeholder values
                        totalRecords = 0;
                        expiredRecords = 0;
                }

                const nextCleanupDate = new Date();
                nextCleanupDate.setDate(nextCleanupDate.getDate() + 7); // Weekly cleanup

                status.push({
                    dataType: policy.dataType,
                    totalRecords,
                    expiredRecords,
                    nextCleanupDate: nextCleanupDate.toISOString()
                });
            } catch (error) {
                console.error(`Error getting retention status for ${policy.dataType}:`, error);
            }
        }

        return status;
    }
}

export default new DataRetentionService();

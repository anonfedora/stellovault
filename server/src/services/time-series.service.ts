import { prisma } from "./database.service";

type TimeSeriesData = {
    timestamp: string;
    value: number;
    metadata?: Record<string, any>;
};

type AggregationPeriod = 'minute' | 'hour' | 'day' | 'week' | 'month';

class TimeSeriesService {
    async aggregateVolumeData(
        startDate: Date,
        endDate: Date,
        period: AggregationPeriod
    ): Promise<TimeSeriesData[]> {
        const timeFormat = this.getTimeFormat(period);
        const groupingColumn = this.getGroupingColumn(period);

        try {
            const escrowData = await prisma.$queryRaw<Array<{ period: string; volume: any }>>`
                SELECT 
                    DATE_TRUNC(${groupingColumn}, "createdAt")::text as period,
                    COALESCE(SUM("amount"), 0) as volume
                FROM "Escrow"
                WHERE "createdAt" >= ${startDate} AND "createdAt" <= ${endDate}
                GROUP BY DATE_TRUNC(${groupingColumn}, "createdAt")
                ORDER BY period ASC
            `;

            const loanData = await prisma.$queryRaw<Array<{ period: string; volume: any }>>`
                SELECT 
                    DATE_TRUNC(${groupingColumn}, "createdAt")::text as period,
                    COALESCE(SUM("amount"), 0) as volume
                FROM "Loan"
                WHERE "createdAt" >= ${startDate} AND "createdAt" <= ${endDate}
                GROUP BY DATE_TRUNC(${groupingColumn}, "createdAt")
                ORDER BY period ASC
            `;

            return this.mergeTimeSeriesData(escrowData, loanData, 'volume');
        } catch (error) {
            console.error('Error aggregating volume data:', error);
            throw error;
        }
    }

    async aggregateUserActivity(
        startDate: Date,
        endDate: Date,
        period: AggregationPeriod
    ): Promise<TimeSeriesData[]> {
        const groupingColumn = this.getGroupingColumn(period);

        try {
            const userActivity = await prisma.$queryRaw<Array<{ period: string; count: any }>>`
                SELECT 
                    DATE_TRUNC(${groupingColumn}, "createdAt")::text as period,
                    COUNT(*) as count
                FROM "User"
                WHERE "createdAt" >= ${startDate} AND "createdAt" <= ${endDate}
                GROUP BY DATE_TRUNC(${groupingColumn}, "createdAt")
                ORDER BY period ASC
            `;

            return userActivity.map((row: { period: string; count: any }) => ({
                timestamp: row.period,
                value: Number(row.count)
            }));
        } catch (error) {
            console.error('Error aggregating user activity:', error);
            throw error;
        }
    }

    async aggregateTransactionCounts(
        startDate: Date,
        endDate: Date,
        period: AggregationPeriod
    ): Promise<TimeSeriesData[]> {
        const groupingColumn = this.getGroupingColumn(period);

        try {
            const escrowCounts = await prisma.$queryRaw<Array<{ period: string; count: any }>>`
                SELECT 
                    DATE_TRUNC(${groupingColumn}, "createdAt")::text as period,
                    COUNT(*) as count
                FROM "Escrow"
                WHERE "createdAt" >= ${startDate} AND "createdAt" <= ${endDate}
                GROUP BY DATE_TRUNC(${groupingColumn}, "createdAt")
                ORDER BY period ASC
            `;

            const loanCounts = await prisma.$queryRaw<Array<{ period: string; count: any }>>`
                SELECT 
                    DATE_TRUNC(${groupingColumn}, "createdAt")::text as period,
                    COUNT(*) as count
                FROM "Loan"
                WHERE "createdAt" >= ${startDate} AND "createdAt" <= ${endDate}
                GROUP BY DATE_TRUNC(${groupingColumn}, "createdAt")
                ORDER BY period ASC
            `;

            return this.mergeTimeSeriesData(escrowCounts, loanCounts, 'count');
        } catch (error) {
            console.error('Error aggregating transaction counts:', error);
            throw error;
        }
    }

    async getPerformanceMetrics(
        startDate: Date,
        endDate: Date,
        period: AggregationPeriod
    ): Promise<TimeSeriesData[]> {
        // Placeholder for performance metrics aggregation
        // In production, this would query actual performance monitoring data
        const intervals = this.getTimeIntervals(startDate, endDate, period);
        
        return intervals.map((timestamp, index) => ({
            timestamp,
            value: Math.random() * 100, // Placeholder performance score
            metadata: {
                cpu: Math.random() * 100,
                memory: Math.random() * 100,
                responseTime: 100 + Math.random() * 200
            }
        }));
    }

    async getComplianceMetrics(
        startDate: Date,
        endDate: Date,
        period: AggregationPeriod
    ): Promise<TimeSeriesData[]> {
        const groupingColumn = this.getGroupingColumn(period);

        try {
            const kycCompliance = await prisma.$queryRaw<Array<{ period: string; rate: any }>>`
                SELECT 
                    DATE_TRUNC(${groupingColumn}, "verifiedAt")::text as period,
                    COUNT(*) as rate
                FROM "Wallet"
                WHERE "verifiedAt" >= ${startDate} AND "verifiedAt" <= ${endDate}
                GROUP BY DATE_TRUNC(${groupingColumn}, "verifiedAt")
                ORDER BY period ASC
            `;

            const totalWallets = await prisma.$queryRaw<Array<{ period: string; total: any }>>`
                SELECT 
                    DATE_TRUNC(${groupingColumn}, "createdAt")::text as period,
                    COUNT(*) as total
                FROM "Wallet"
                WHERE "createdAt" >= ${startDate} AND "createdAt" <= ${endDate}
                GROUP BY DATE_TRUNC(${groupingColumn}, "createdAt")
                ORDER BY period ASC
            `;

            return this.calculateComplianceRates(kycCompliance, totalWallets);
        } catch (error) {
            console.error('Error aggregating compliance metrics:', error);
            throw error;
        }
    }

    async generateTimeSeriesReport(
        type: 'volume' | 'users' | 'transactions' | 'performance' | 'compliance',
        startDate: Date,
        endDate: Date,
        period: AggregationPeriod
    ): Promise<TimeSeriesData[]> {
        switch (type) {
            case 'volume':
                return this.aggregateVolumeData(startDate, endDate, period);
            case 'users':
                return this.aggregateUserActivity(startDate, endDate, period);
            case 'transactions':
                return this.aggregateTransactionCounts(startDate, endDate, period);
            case 'performance':
                return this.getPerformanceMetrics(startDate, endDate, period);
            case 'compliance':
                return this.getComplianceMetrics(startDate, endDate, period);
            default:
                throw new Error(`Unknown time series type: ${type}`);
        }
    }

    // Helper methods
    private getTimeFormat(period: AggregationPeriod): string {
        const formats = {
            minute: 'YYYY-MM-DD HH24:MI:00',
            hour: 'YYYY-MM-DD HH24:00:00',
            day: 'YYYY-MM-DD',
            week: 'YYYY-"W"WW',
            month: 'YYYY-MM'
        };
        return formats[period];
    }

    private getGroupingColumn(period: AggregationPeriod): string {
        const columns = {
            minute: 'minute',
            hour: 'hour',
            day: 'day',
            week: 'week',
            month: 'month'
        };
        return columns[period];
    }

    private mergeTimeSeriesData(
        dataset1: Array<{ period: string; [key: string]: any }>,
        dataset2: Array<{ period: string; [key: string]: any }>,
        valueKey: string
    ): TimeSeriesData[] {
        const merged = new Map<string, number>();

        // Process first dataset
        dataset1.forEach(row => {
            merged.set(row.period, Number(row[valueKey]));
        });

        // Add second dataset
        dataset2.forEach(row => {
            const existing = merged.get(row.period) || 0;
            merged.set(row.period, existing + Number(row[valueKey]));
        });

        return Array.from(merged.entries()).map(([timestamp, value]) => ({
            timestamp,
            value
        })).sort((a, b) => a.timestamp.localeCompare(b.timestamp));
    }

    private calculateComplianceRates(
        kycData: Array<{ period: string; rate: any }>,
        totalData: Array<{ period: string; total: any }>
    ): TimeSeriesData[] {
        const kycMap = new Map(kycData.map(row => [row.period, Number(row.rate)]));
        const totalMap = new Map(totalData.map(row => [row.period, Number(row.total)]));

        const result: TimeSeriesData[] = [];

        totalMap.forEach((total, period) => {
            const kyc = kycMap.get(period) || 0;
            const complianceRate = total > 0 ? (kyc / total) * 100 : 0;
            
            result.push({
                timestamp: period,
                value: complianceRate
            });
        });

        return result.sort((a, b) => a.timestamp.localeCompare(b.timestamp));
    }

    private getTimeIntervals(startDate: Date, endDate: Date, period: AggregationPeriod): string[] {
        const intervals: string[] = [];
        const current = new Date(startDate);

        while (current <= endDate) {
            intervals.push(current.toISOString());
            
            switch (period) {
                case 'minute':
                    current.setMinutes(current.getMinutes() + 1);
                    break;
                case 'hour':
                    current.setHours(current.getHours() + 1);
                    break;
                case 'day':
                    current.setDate(current.getDate() + 1);
                    break;
                case 'week':
                    current.setDate(current.getDate() + 7);
                    break;
                case 'month':
                    current.setMonth(current.getMonth() + 1);
                    break;
            }
        }

        return intervals;
    }

    async exportTimeSeriesData(
        data: TimeSeriesData[],
        format: 'json' | 'csv'
    ): Promise<string> {
        if (format === 'json') {
            return JSON.stringify(data, null, 2);
        }

        // CSV format
        const headers = ['timestamp', 'value'];
        const rows = data.map(d => [d.timestamp, d.value.toString()]);
        
        return [headers, ...rows]
            .map(row => row.join(','))
            .join('\n');
    }
}

export default new TimeSeriesService();

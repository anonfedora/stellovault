import client from "prom-client";
import { Request, Response, NextFunction } from "express";
import blockchainService from "./blockchain.service";
import { env } from "../config/env";

// Initialize Prometheus registry
const register = new client.Registry();
client.collectDefaultMetrics({ register });

// --- Metrics Definitions ---

// API Latency
const httpRequestDurationMicroseconds = new client.Histogram({
    name: "http_request_duration_seconds",
    help: "Duration of HTTP requests in seconds",
    labelNames: ["method", "route", "status_code"],
    buckets: [0.1, 0.3, 0.5, 0.7, 1, 3, 5, 7, 10], // seconds
});

// Request Count
const httpRequestsTotal = new client.Counter({
    name: "http_requests_total",
    help: "Total number of HTTP requests",
    labelNames: ["method", "route", "status_code"],
});

// Server Health
const serverHealthStatus = new client.Gauge({
    name: "server_health_status",
    help: "Server health status (1 = OK, 0 = Error)",
});

// Fee Payer Balance
const feePayerBalance = new client.Gauge({
    name: "fee_payer_balance_xlm",
    help: "Current balance of the Fee Payer account in XLM",
});

// Register metrics
register.registerMetric(httpRequestDurationMicroseconds);
register.registerMetric(httpRequestsTotal);
register.registerMetric(serverHealthStatus);
register.registerMetric(feePayerBalance);

class MetricsService {
    public getRegistry() {
        return register;
    }

    /**
     * Middleware to track HTTP request metrics
     */
    public metricsMiddleware(req: Request, res: Response, next: NextFunction) {
        const start = process.hrtime();

        res.on("finish", () => {
            const duration = process.hrtime(start);
            const durationInSeconds = duration[0] + duration[1] / 1e9;
            const route = req.route?.path || req.path;
            const status = res.statusCode.toString();

            httpRequestDurationMicroseconds
                .labels(req.method, route, status)
                .observe(durationInSeconds);

            httpRequestsTotal
                .labels(req.method, route, status)
                .inc();
        });

        next();
    }

    /**
     * Start background monitoring tasks
     */
    public startBackgroundMonitoring() {
        console.log("Starting background monitoring...");
        
        // Initial health check
        serverHealthStatus.set(1);

        // Check fee payer balance every 5 minutes
        this.checkFeePayerBalance();
        setInterval(() => this.checkFeePayerBalance(), 5 * 60 * 1000);
    }

    private async checkFeePayerBalance() {
        try {
            const publicKey = env.feePayer.publicKey;
            if (!publicKey) {
                console.warn("Fee Payer public key not configured, skipping balance check.");
                return;
            }

            const balance = await blockchainService.getAccountBalance(publicKey, "native");
            const balanceNum = parseFloat(balance);
            feePayerBalance.set(balanceNum);

            // Alerting logic: if balance is low (e.g., < 10 XLM)
            if (balanceNum < 10) {
                console.error(`ALERT: Fee Payer balance is low: ${balanceNum} XLM (Threshold: 10 XLM)`);
                // In a real app, you might send a Slack/Email/Telegram alert here
            }
        } catch (error) {
            console.error("Failed to check fee payer balance:", error);
        }
    }
}

export default new MetricsService();

import express, { Request, Response } from "express";
import cors from "cors";
import helmet from "helmet";
import morgan from "morgan";
import { createServer } from "http";
import WebSocket from "ws";
import { env } from "./config/env";
import websocketService, { WsState } from "./services/websocket.service";
import transactionQueueService from "./services/transaction-queue.service";

// Routes
import authRoutes from "./routes/auth.routes";
import walletRoutes from "./routes/wallet.routes";
import userRoutes from "./routes/user.routes";
import escrowRoutes from "./routes/escrow.routes";
import collateralRoutes from "./routes/collateral.routes";
import loanRoutes from "./routes/loan.routes";
import oracleRoutes from "./routes/oracle.routes";
import confirmationRoutes from "./routes/confirmation.routes";
import governanceRoutes from "./routes/governance.routes";
import riskRoutes from "./routes/risk.routes";
import analyticsRoutes from "./routes/analytics.routes";
import collateralService from "./services/collateral.service";
import metricsService from "./services/metrics.service";
import webhookService from "./services/webhook.service";

// Middleware
import {
  geoIpBlockMiddleware,
  tieredRateLimitMiddleware,
} from "./middleware/rate-limit.middleware";
import {
  errorMiddleware,
  notFoundMiddleware,
} from "./middleware/error.middleware";
import { requestTraceMiddleware } from "./middleware/request-trace.middleware";
import paymentRoutes from "./routes/payment.routes";

const app = express();

// ── Global Middleware ────────────────────────────────────────────────────────
app.use(helmet());
app.use(requestTraceMiddleware);
app.use(cors({ origin: env.corsAllowedOrigins }));
app.use(morgan("dev"));
app.use(
  express.json({
    verify: (req, _res, buf) => {
      (req as Request & { rawBody?: string }).rawBody = buf.toString("utf8");
    },
  }),
);
app.use(metricsService.metricsMiddleware.bind(metricsService));
app.use(geoIpBlockMiddleware);
app.use(tieredRateLimitMiddleware);

// ── Health ───────────────────────────────────────────────────────────────────
app.get("/health", (_req: Request, res: Response) => {
  res.json({ status: "ok", version: "1.0.0", timestamp: new Date() });
});

app.get("/metrics", async (_req: Request, res: Response) => {
  res.set("Content-Type", metricsService.getRegistry().contentType);
  res.end(await metricsService.getRegistry().metrics());
});

// ── API Routes ───────────────────────────────────────────────────────────────
const api = "/api";

app.use("/api", paymentRoutes);
app.use(`${api}/auth`, authRoutes);
app.use(`${api}/wallets`, walletRoutes);
app.use(`${api}/users`, userRoutes);
app.use(`${api}/escrows`, escrowRoutes);
app.use(`${api}/collateral`, collateralRoutes);
app.use(`${api}/loans`, loanRoutes);
app.use(`${api}/oracles`, oracleRoutes);
app.use(`${api}/confirmations`, confirmationRoutes);
app.use(`${api}/governance`, governanceRoutes);
app.use(`${api}/risk`, riskRoutes);
app.use(`${api}/analytics`, analyticsRoutes);
app.use(`${api}/v1/analytics`, analyticsRoutes);

// ── Error Handling (must be last) ────────────────────────────────────────────
app.use(notFoundMiddleware);
app.use(errorMiddleware);

const port = env.port;
const server = app.listen(port, () => {
  console.log(`StelloVault server running on http://localhost:${port}`);
  console.log(`WebSocket endpoint: ws://localhost:${port}/ws`);
  console.log(`Routes mounted at ${api}`);

  // Start background jobs
  collateralService.startIndexer();
  metricsService.startBackgroundMonitoring();
});

function gracefulShutdown(signal: string) {
  console.log(`Received ${signal}. Shutting down gracefully...`);
  collateralService.stopIndexer();

  // Close transaction queue
  transactionQueueService
    .close()
    .then(() => {
      console.log("Transaction queue closed.");
    })
    .catch((err) => {
      console.error("Error closing transaction queue:", err);
    });
  webhookService
    .close()
    .then(() => {
      console.log("Webhook worker closed.");
    })
    .catch((err) => {
      console.error("Error closing webhook worker:", err);
    });

  server.close(() => {
    console.log("Server closed.");
    process.exit(0);
  });

  setTimeout(() => {
    console.error(
      "Could not close connections in time, forcefully shutting down",
    );
    process.exit(1);
  }, 10000).unref();
}

process.on("SIGTERM", () => gracefulShutdown("SIGTERM"));
process.on("SIGINT", () => gracefulShutdown("SIGINT"));

export default app;

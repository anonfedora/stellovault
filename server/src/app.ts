import express, { Request, Response } from "express";
import cors from "cors";
import helmet from "helmet";
import morgan from "morgan";
import { env } from "./config/env";
import logger from "./config/logger";

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
import paymentRoutes from "./routes/payment.routes";

import collateralService from "./services/collateral.service";
import metricsService from "./services/metrics.service";
import { prisma } from "./services/database.service";

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

const app = express();
const api = "/api";
const version = process.env.npm_package_version || "1.0.0";

app.use(helmet());
app.use(requestTraceMiddleware);
app.use(cors({ origin: env.corsAllowedOrigins }));
app.use(
  morgan("combined", {
    stream: {
      write: (message) => logger.info("http_access", { message: message.trim() }),
    },
  }),
);
app.use((req, res, next) => {
  req.setTimeout(env.requestTimeoutMs);
  res.setTimeout(env.requestTimeoutMs);
  next();
});
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

app.get("/health", async (_req: Request, res: Response) => {
  try {
    await prisma.$queryRaw`SELECT 1`;
    res.json({ status: "ok", database: "connected", version });
  } catch (error) {
    logger.error("health_database_check_failed", { error });
    res.status(503).json({ status: "error", database: "disconnected", version });
  }
});

app.get("/metrics", async (_req: Request, res: Response) => {
  res.set("Content-Type", metricsService.getRegistry().contentType);
  res.end(await metricsService.getRegistry().metrics());
});

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

app.use(notFoundMiddleware);
app.use(errorMiddleware);

export function startBackgroundJobs() {
  collateralService.startIndexer();
  metricsService.startBackgroundMonitoring();
}

export function stopBackgroundJobs() {
  collateralService.stopIndexer();
}

export default app;

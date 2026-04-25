import { env } from "./config/env";
import logger from "./config/logger";
import app, { startBackgroundJobs, stopBackgroundJobs } from "./app";
import transactionQueueService from "./services/transaction-queue.service";
import webhookService from "./services/webhook.service";
import { prisma } from "./services/database.service";

const server = app.listen(env.port, () => {
  logger.info("server_started", {
    port: env.port,
    url: `http://localhost:${env.port}`,
    websocketUrl: `ws://localhost:${env.port}/ws`,
  });

  startBackgroundJobs();
});

async function gracefulShutdown(signal: string) {
  logger.info("server_shutdown_started", { signal });
  stopBackgroundJobs();

  const closeHttpServer = new Promise<void>((resolve, reject) => {
    server.close((error) => {
      if (error) reject(error);
      else resolve();
    });
  });

  const forceExit = setTimeout(() => {
    logger.error("server_shutdown_timeout");
    process.exit(1);
  }, 10000);
  forceExit.unref();

  try {
    await Promise.all([
      closeHttpServer,
      transactionQueueService.close(),
      webhookService.close(),
      prisma.$disconnect(),
    ]);
    clearTimeout(forceExit);
    logger.info("server_shutdown_complete");
    process.exit(0);
  } catch (error) {
    clearTimeout(forceExit);
    logger.error("server_shutdown_failed", { error });
    process.exit(1);
  }
}

process.on("SIGTERM", () => void gracefulShutdown("SIGTERM"));
process.on("SIGINT", () => void gracefulShutdown("SIGINT"));

import { randomUUID } from "crypto";
import { Queue, QueueEvents, Worker } from "bullmq";
import { Redis } from "ioredis";
import { env } from "../config/env";
import { signWebhookPayload } from "./webhook-signature.service";

interface PaymentWebhookJobData {
  loanId: string;
  repaymentId: string;
  paymentSessionId: string;
  webhookUrl: string;
  webhookSecret: string;
  payload: Record<string, unknown>;
}

export class WebhookService {
  private readonly connection: Redis;
  private readonly queue: Queue<PaymentWebhookJobData>;
  private readonly queueEvents: QueueEvents;
  private readonly worker: Worker<PaymentWebhookJobData>;

  constructor() {
    this.connection = new Redis({
      host: env.redis.host,
      port: env.redis.port,
      maxRetriesPerRequest: null,
    });

    this.queue = new Queue<PaymentWebhookJobData>("payment-webhooks", {
      connection: this.connection,
      defaultJobOptions: {
        attempts: 5,
        backoff: {
          type: "exponential",
          delay: 1000,
        },
        removeOnComplete: {
          age: 24 * 3600,
          count: 1000,
        },
        removeOnFail: false,
      },
    });

    this.queueEvents = new QueueEvents("payment-webhooks", {
      connection: this.connection,
    });

    this.worker = new Worker<PaymentWebhookJobData>(
      "payment-webhooks",
      async (job) => {
        await this.sendWebhook(job.data);
      },
      {
        connection: this.connection,
        concurrency: 5,
      },
    );

    this.worker.on("failed", (job, error) => {
      console.error(
        `Payment webhook job ${job?.id ?? "unknown"} failed:`,
        error,
      );
    });

    this.queueEvents.on("failed", ({ jobId, failedReason }) => {
      console.error(
        `Payment webhook queue event failed for ${jobId}: ${failedReason}`,
      );
    });
  }

  async enqueuePaymentReceivedWebhook(
    data: PaymentWebhookJobData,
  ): Promise<string> {
    const jobId = `${data.paymentSessionId}:${data.repaymentId}`;
    const job = await this.queue.add("payment-received", data, { jobId });
    return job.id ?? jobId;
  }

  async sendWebhook(data: PaymentWebhookJobData): Promise<void> {
    const body = JSON.stringify(data.payload);
    const signature = signWebhookPayload(body, data.webhookSecret);
    const response = await fetch(data.webhookUrl, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-webhook-event": "payment.received",
        "x-webhook-id": randomUUID(),
        "x-webhook-signature": signature,
      },
      body,
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(
        `Webhook delivery failed with status ${response.status}: ${errorText || response.statusText}`,
      );
    }
  }

  async close(): Promise<void> {
    await this.worker.close();
    await this.queue.close();
    await this.queueEvents.close();
    await this.connection.quit();
  }
}

export default new WebhookService();

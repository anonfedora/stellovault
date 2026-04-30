import {
  createCipheriv,
  createHash,
  randomBytes,
  randomUUID,
} from "crypto";
import { Queue, QueueEvents, Worker } from "bullmq";
import { Redis } from "ioredis";
import { env } from "../config/env";
import { NotFoundError, ValidationError } from "../config/errors";
import { prisma } from "../config/prisma";
import {
  signWebhookPayload,
  verifyWebhookSignature as verifySignature,
} from "./webhook-signature.service";

type WebhookAuthMethod =
  | "SIGNATURE"
  | "BEARER"
  | "BASIC"
  | "CUSTOM_HEADER"
  | "NONE";

interface WebhookRecord {
  id: string;
  userId: string;
  url: string;
  secret: string;
  events: string[];
  isActive: boolean;
  authMethod: WebhookAuthMethod;
  authConfig?: Record<string, unknown> | null;
  encryptionEnabled: boolean;
  encryptionKey?: string | null;
  rateLimitPerMinute: number;
  lastSuccessAt?: Date | null;
  lastFailureAt?: Date | null;
  consecutiveFailures: number;
}

interface WebhookDeliveryRecord {
  id: string;
  webhookId: string;
  eventType: string;
  status: "PENDING" | "RETRYING" | "SUCCESS" | "FAILED";
  attempt: number;
  requestBody?: Record<string, unknown> | null;
  responseStatus?: number | null;
  responseBody?: string | null;
  errorMessage?: string | null;
}

interface RegisterWebhookOptions {
  authMethod?: WebhookAuthMethod;
  authConfig?: Record<string, unknown>;
  encryptionEnabled?: boolean;
  encryptionKey?: string;
  rateLimitPerMinute?: number;
  isActive?: boolean;
}

interface UpdateWebhookInput extends RegisterWebhookOptions {
  url?: string;
  events?: string[];
  secret?: string;
}

interface TriggerEventOptions {
  userId?: string;
}

interface WebhookJobData {
  deliveryId?: string;
  webhookId?: string;
  eventType: string;
  payload: Record<string, unknown>;
  directDelivery?: {
    webhookUrl: string;
    webhookSecret: string;
  };
}

interface PaymentWebhookJobData {
  loanId: string;
  repaymentId: string;
  paymentSessionId: string;
  webhookUrl: string;
  webhookSecret: string;
  payload: Record<string, unknown>;
}

const WEBHOOK_QUEUE_NAME = "webhook-delivery";
const MAX_RETRY_ATTEMPTS = 5;
const RETRY_BASE_DELAY_MS = 1000;
const RETRY_MAX_DELAY_MS = 5 * 60 * 1000;
const WEBHOOK_AUTH_METHODS: WebhookAuthMethod[] = [
  "SIGNATURE",
  "BEARER",
  "BASIC",
  "CUSTOM_HEADER",
  "NONE",
];

export class WebhookService {
  private readonly connection: Redis;
  private readonly queue: Queue<WebhookJobData>;
  private readonly queueEvents: QueueEvents;
  private readonly worker: Worker<WebhookJobData>;
  private readonly rateWindow = new Map<
    string,
    { windowStart: number; count: number }
  >();

  constructor() {
    this.connection = new Redis({
      host: env.redis.host,
      port: env.redis.port,
      maxRetriesPerRequest: null,
    });

    this.queue = new Queue<WebhookJobData>(WEBHOOK_QUEUE_NAME, {
      connection: this.connection,
      defaultJobOptions: {
        attempts: 1,
        removeOnComplete: {
          age: 24 * 3600,
          count: 1000,
        },
        removeOnFail: {
          age: 7 * 24 * 3600,
        },
      },
    });

    this.queueEvents = new QueueEvents(WEBHOOK_QUEUE_NAME, {
      connection: this.connection,
    });

    this.worker = new Worker<WebhookJobData>(
      WEBHOOK_QUEUE_NAME,
      async (job) => this.processWebhookJob(job.data),
      {
        connection: this.connection,
        concurrency: 10,
        limiter: {
          max: 300,
          duration: 1000,
        },
      },
    );

    this.worker.on("failed", (job, error) => {
      console.error(`Webhook worker failed for job ${job?.id ?? "unknown"}`, error);
    });

    this.queueEvents.on("failed", ({ jobId, failedReason }) => {
      console.error(`Webhook queue event failed for ${jobId}: ${failedReason}`);
    });
  }

  async registerWebhook(
    userId: string,
    url: string,
    events: string[],
    secret?: string,
    options: RegisterWebhookOptions = {},
  ): Promise<WebhookRecord> {
    this.validateWebhookUrl(url);
    await this.validateEndpointAccessibility(url);

    const normalizedEvents = this.normalizeEvents(events);
    const normalizedSecret = secret?.trim() || randomBytes(32).toString("hex");
    const authMethod = this.normalizeAuthMethod(options.authMethod);
    const rateLimitPerMinute = this.normalizeRateLimit(options.rateLimitPerMinute);
    const encryptionEnabled = options.encryptionEnabled ?? false;
    const encryptionKey = options.encryptionKey?.trim() || null;

    if (encryptionEnabled && !encryptionKey) {
      throw new ValidationError(
        "encryptionKey is required when encryptionEnabled is true",
      );
    }

    const webhook = await this.prisma().webhook.create({
      data: {
        userId,
        url,
        secret: normalizedSecret,
        events: normalizedEvents,
        authMethod,
        authConfig: options.authConfig ?? null,
        encryptionEnabled,
        encryptionKey,
        rateLimitPerMinute,
        isActive: options.isActive ?? true,
      },
    });

    return webhook as WebhookRecord;
  }

  async listWebhooks(userId: string): Promise<WebhookRecord[]> {
    const webhooks = await this.prisma().webhook.findMany({
      where: { userId },
      orderBy: { createdAt: "desc" },
    });
    return webhooks as WebhookRecord[];
  }

  async updateWebhook(
    userId: string,
    webhookId: string,
    input: UpdateWebhookInput,
  ): Promise<WebhookRecord> {
    const webhook = await this.getOwnedWebhook(userId, webhookId);

    const data: Record<string, unknown> = {};

    if (typeof input.url === "string") {
      this.validateWebhookUrl(input.url);
      await this.validateEndpointAccessibility(input.url);
      data.url = input.url;
    }

    if (Array.isArray(input.events)) {
      data.events = this.normalizeEvents(input.events);
    }

    if (typeof input.secret === "string") {
      const secret = input.secret.trim();
      if (!secret) {
        throw new ValidationError("secret cannot be empty");
      }
      data.secret = secret;
    }

    if (typeof input.isActive === "boolean") {
      data.isActive = input.isActive;
    }

    if (typeof input.authMethod === "string") {
      data.authMethod = this.normalizeAuthMethod(input.authMethod);
    }

    if (typeof input.authConfig !== "undefined") {
      data.authConfig = input.authConfig;
    }

    if (typeof input.rateLimitPerMinute !== "undefined") {
      data.rateLimitPerMinute = this.normalizeRateLimit(input.rateLimitPerMinute);
    }

    const encryptionEnabled =
      typeof input.encryptionEnabled === "boolean"
        ? input.encryptionEnabled
        : webhook.encryptionEnabled;
    const encryptionKey =
      typeof input.encryptionKey === "string"
        ? input.encryptionKey.trim()
        : webhook.encryptionKey;

    if (encryptionEnabled && !encryptionKey) {
      throw new ValidationError(
        "encryptionKey is required when encryptionEnabled is true",
      );
    }

    data.encryptionEnabled = encryptionEnabled;
    data.encryptionKey = encryptionKey ?? null;

    const updated = await this.prisma().webhook.update({
      where: { id: webhookId },
      data,
    });

    return updated as WebhookRecord;
  }

  async deleteWebhook(userId: string, webhookId: string): Promise<void> {
    const result = await this.prisma().webhook.deleteMany({
      where: { id: webhookId, userId },
    });

    if (result.count === 0) {
      throw new NotFoundError("Webhook not found");
    }
  }

  async testWebhook(userId: string, webhookId: string): Promise<string> {
    const webhook = await this.getOwnedWebhook(userId, webhookId);
    const payload = {
      event: "webhook.test",
      webhookId: webhook.id,
      sentAt: new Date().toISOString(),
      message: "Test webhook delivery from StelloVault",
    };

    return this.queueWebhookDelivery(webhook, "webhook.test", payload);
  }

  async getWebhookLogs(
    userId: string,
    webhookId: string,
    limit = 50,
  ): Promise<WebhookDeliveryRecord[]> {
    await this.getOwnedWebhook(userId, webhookId);

    const logs = await this.prisma().webhookDelivery.findMany({
      where: { webhookId },
      orderBy: { createdAt: "desc" },
      take: Math.max(1, Math.min(limit, 200)),
    });

    return logs as WebhookDeliveryRecord[];
  }

  async triggerEvent(
    eventType: string,
    data: Record<string, unknown>,
    options: TriggerEventOptions = {},
  ): Promise<{ queued: number; webhookIds: string[] }> {
    const normalizedEventType = eventType.trim().toLowerCase();
    if (!normalizedEventType) {
      throw new ValidationError("eventType is required");
    }

    const where: Record<string, unknown> = {
      isActive: true,
      events: { has: normalizedEventType },
    };

    if (options.userId) {
      where.userId = options.userId;
    }

    const webhooks = (await this.prisma().webhook.findMany({
      where,
    })) as WebhookRecord[];

    const webhookIds: string[] = [];
    for (const webhook of webhooks) {
      await this.queueWebhookDelivery(webhook, normalizedEventType, {
        event: normalizedEventType,
        timestamp: new Date().toISOString(),
        data,
      });
      webhookIds.push(webhook.id);
    }

    return { queued: webhookIds.length, webhookIds };
  }

  async retryFailedWebhooks(): Promise<number> {
    const now = new Date();
    const deliveries = (await this.prisma().webhookDelivery.findMany({
      where: {
        OR: [
          {
            status: "RETRYING",
            nextRetryAt: { lte: now },
          },
          {
            status: "FAILED",
            nextRetryAt: { lte: now },
          },
        ],
      },
      orderBy: { createdAt: "asc" },
      take: 200,
    })) as WebhookDeliveryRecord[];

    for (const delivery of deliveries) {
      await this.prisma().webhookDelivery.update({
        where: { id: delivery.id },
        data: {
          status: "RETRYING",
        },
      });

      await this.queue.add(
        `retry:${delivery.id}`,
        {
          deliveryId: delivery.id,
          webhookId: delivery.webhookId,
          eventType: delivery.eventType,
          payload: (delivery.requestBody as Record<string, unknown>) ?? {},
        },
        {
          jobId: `retry:${delivery.id}:${Date.now()}`,
        },
      );
    }

    return deliveries.length;
  }

  verifyWebhookSignature(
    payload: string,
    signature: string | undefined,
    secret: string,
  ): boolean {
    return verifySignature(payload, signature, secret);
  }

  async getWebhookMetrics(webhookId: string, userId?: string) {
    if (userId) {
      await this.getOwnedWebhook(userId, webhookId);
    }

    const [total, successful, failed, pending, retrying, durationAgg, latest] =
      await Promise.all([
        this.prisma().webhookDelivery.count({ where: { webhookId } }),
        this.prisma().webhookDelivery.count({
          where: { webhookId, status: "SUCCESS" },
        }),
        this.prisma().webhookDelivery.count({
          where: { webhookId, status: "FAILED" },
        }),
        this.prisma().webhookDelivery.count({
          where: { webhookId, status: "PENDING" },
        }),
        this.prisma().webhookDelivery.count({
          where: { webhookId, status: "RETRYING" },
        }),
        this.prisma().webhookDelivery.aggregate({
          where: { webhookId, status: "SUCCESS" },
          _avg: { durationMs: true },
        }),
        this.prisma().webhookDelivery.findMany({
          where: { webhookId },
          orderBy: { createdAt: "desc" },
          take: 20,
          select: { status: true },
        }),
      ]);

    const successRate = total === 0 ? 0 : Number(((successful / total) * 100).toFixed(2));
    const health =
      latest.length === 0
        ? "unknown"
        : latest.slice(0, 5).every((entry: { status: string }) => entry.status === "SUCCESS")
          ? "healthy"
          : latest.slice(0, 3).every((entry: { status: string }) => entry.status !== "SUCCESS")
            ? "degraded"
            : "warning";

    return {
      webhookId,
      totalDeliveries: total,
      successfulDeliveries: successful,
      failedDeliveries: failed,
      pendingDeliveries: pending,
      retryingDeliveries: retrying,
      successRate,
      averageDeliveryMs: durationAgg?._avg?.durationMs
        ? Number(durationAgg._avg.durationMs)
        : null,
      health,
    };
  }

  async enqueuePaymentReceivedWebhook(
    data: PaymentWebhookJobData,
  ): Promise<string> {
    const job = await this.queue.add(
      "payment-received-direct",
      {
        eventType: "payment.received",
        payload: data.payload,
        directDelivery: {
          webhookUrl: data.webhookUrl,
          webhookSecret: data.webhookSecret,
        },
      },
      {
        jobId: `payment:${data.paymentSessionId}:${data.repaymentId}`,
      },
    );
    return job.id ?? `payment:${data.paymentSessionId}:${data.repaymentId}`;
  }

  async sendWebhook(data: PaymentWebhookJobData): Promise<void> {
    await this.sendDirectWebhook(
      data.webhookUrl,
      data.webhookSecret,
      "payment.received",
      data.payload,
    );
  }

  async close(): Promise<void> {
    await this.worker.close();
    await this.queue.close();
    await this.queueEvents.close();
    await this.connection.quit();
  }

  private prisma() {
    return prisma as any;
  }

  private async processWebhookJob(jobData: WebhookJobData): Promise<void> {
    if (jobData.directDelivery) {
      await this.sendDirectWebhook(
        jobData.directDelivery.webhookUrl,
        jobData.directDelivery.webhookSecret,
        jobData.eventType,
        jobData.payload,
      );
      return;
    }

    if (!jobData.deliveryId) {
      return;
    }

    const delivery = (await this.prisma().webhookDelivery.findUnique({
      where: { id: jobData.deliveryId },
      include: { webhook: true },
    })) as (WebhookDeliveryRecord & { webhook?: WebhookRecord }) | null;

    if (!delivery || !delivery.webhook) {
      return;
    }

    if (!delivery.webhook.isActive) {
      await this.prisma().webhookDelivery.update({
        where: { id: delivery.id },
        data: {
          status: "FAILED",
          errorMessage: "Webhook disabled",
          nextRetryAt: null,
        },
      });
      return;
    }

    const rateDelay = this.getRateLimitDelayMs(delivery.webhook);
    if (rateDelay > 0) {
      await this.prisma().webhookDelivery.update({
        where: { id: delivery.id },
        data: {
          status: "RETRYING",
          errorMessage: `Rate limited. Retrying in ${rateDelay}ms`,
          nextRetryAt: new Date(Date.now() + rateDelay),
        },
      });

      await this.queue.add(
        `rate-limit:${delivery.id}`,
        {
          deliveryId: delivery.id,
          webhookId: delivery.webhook.id,
          eventType: delivery.eventType,
          payload: (delivery.requestBody as Record<string, unknown>) ?? jobData.payload,
        },
        {
          jobId: `rate-limit:${delivery.id}:${Date.now()}`,
          delay: rateDelay,
        },
      );
      return;
    }

    const startedAt = Date.now();
    const attempt = delivery.attempt + 1;
    const requestPayload =
      (delivery.requestBody as Record<string, unknown>) ?? jobData.payload;

    try {
      const request = this.buildWebhookRequest(
        delivery.webhook,
        delivery.eventType,
        requestPayload,
      );
      const response = await this.fetchWithTimeout(delivery.webhook.url, {
        method: "POST",
        headers: request.headers,
        body: request.body,
      });
      const responseBody = await response.text();

      if (!response.ok) {
        throw new Error(
          `HTTP ${response.status}${responseBody ? `: ${responseBody}` : ""}`,
        );
      }

      const durationMs = Date.now() - startedAt;
      await this.prisma().webhookDelivery.update({
        where: { id: delivery.id },
        data: {
          status: "SUCCESS",
          attempt,
          requestHeaders: request.headers,
          requestBody: request.logPayload,
          responseStatus: response.status,
          responseBody,
          deliveredAt: new Date(),
          durationMs,
          errorMessage: null,
          nextRetryAt: null,
        },
      });

      await this.prisma().webhook.update({
        where: { id: delivery.webhook.id },
        data: {
          lastSuccessAt: new Date(),
          consecutiveFailures: 0,
        },
      });
    } catch (error) {
      await this.handleDeliveryFailure(delivery, attempt, error);
    }
  }

  private async queueWebhookDelivery(
    webhook: WebhookRecord,
    eventType: string,
    payload: Record<string, unknown>,
  ): Promise<string> {
    const delivery = await this.prisma().webhookDelivery.create({
      data: {
        webhookId: webhook.id,
        eventType,
        status: "PENDING",
        requestBody: payload,
      },
    });

    await this.queue.add(
      `delivery:${delivery.id}`,
      {
        deliveryId: delivery.id,
        webhookId: webhook.id,
        eventType,
        payload,
      },
      {
        jobId: `delivery:${delivery.id}`,
      },
    );

    return delivery.id as string;
  }

  private async handleDeliveryFailure(
    delivery: WebhookDeliveryRecord & { webhook?: WebhookRecord },
    attempt: number,
    error: unknown,
  ): Promise<void> {
    const reason = error instanceof Error ? error.message : "Unknown webhook error";
    const now = new Date();

    if (delivery.webhook) {
      await this.prisma().webhook.update({
        where: { id: delivery.webhook.id },
        data: {
          lastFailureAt: now,
          consecutiveFailures: {
            increment: 1,
          },
        },
      });
    }

    if (attempt >= MAX_RETRY_ATTEMPTS) {
      await this.prisma().webhookDelivery.update({
        where: { id: delivery.id },
        data: {
          status: "FAILED",
          attempt,
          errorMessage: reason,
          nextRetryAt: null,
        },
      });
      return;
    }

    const retryDelay = this.calculateRetryDelay(attempt);
    const nextRetryAt = new Date(Date.now() + retryDelay);
    await this.prisma().webhookDelivery.update({
      where: { id: delivery.id },
      data: {
        status: "RETRYING",
        attempt,
        errorMessage: reason,
        nextRetryAt,
      },
    });

    await this.queue.add(
      `retry:${delivery.id}`,
      {
        deliveryId: delivery.id,
        webhookId: delivery.webhookId,
        eventType: delivery.eventType,
        payload: (delivery.requestBody as Record<string, unknown>) ?? {},
      },
      {
        jobId: `retry:${delivery.id}:${attempt}`,
        delay: retryDelay,
      },
    );
  }

  private buildWebhookRequest(
    webhook: WebhookRecord,
    eventType: string,
    payload: Record<string, unknown>,
  ): {
    body: string;
    headers: Record<string, string>;
    logPayload: Record<string, unknown>;
  } {
    const transformedPayload = webhook.encryptionEnabled
      ? this.encryptPayload(payload, webhook.encryptionKey ?? "")
      : payload;

    const body = JSON.stringify(transformedPayload);
    const headers: Record<string, string> = {
      "content-type": "application/json",
      "x-webhook-event": eventType,
      "x-webhook-id": randomUUID(),
      "x-webhook-timestamp": new Date().toISOString(),
    };

    if (webhook.authMethod === "SIGNATURE") {
      headers["x-webhook-signature"] = signWebhookPayload(body, webhook.secret);
    }

    if (webhook.authMethod === "BEARER") {
      const token = typeof webhook.authConfig?.token === "string"
        ? webhook.authConfig.token
        : "";
      if (token) {
        headers.authorization = `Bearer ${token}`;
      }
    }

    if (webhook.authMethod === "BASIC") {
      const username =
        typeof webhook.authConfig?.username === "string"
          ? webhook.authConfig.username
          : "";
      const password =
        typeof webhook.authConfig?.password === "string"
          ? webhook.authConfig.password
          : "";
      if (username || password) {
        headers.authorization = `Basic ${Buffer.from(`${username}:${password}`).toString("base64")}`;
      }
    }

    if (webhook.authMethod === "CUSTOM_HEADER") {
      const headerName =
        typeof webhook.authConfig?.headerName === "string"
          ? webhook.authConfig.headerName
          : "";
      const headerValue =
        typeof webhook.authConfig?.headerValue === "string"
          ? webhook.authConfig.headerValue
          : "";
      if (headerName && headerValue) {
        headers[headerName.toLowerCase()] = headerValue;
      }
    }

    return { body, headers, logPayload: transformedPayload };
  }

  private encryptPayload(
    payload: Record<string, unknown>,
    encryptionKey: string,
  ): Record<string, unknown> {
    if (!encryptionKey) {
      throw new ValidationError("Missing encryption key");
    }

    const iv = randomBytes(12);
    const key = createHash("sha256").update(encryptionKey).digest();
    const cipher = createCipheriv("aes-256-gcm", key, iv);
    const encodedPayload = Buffer.from(JSON.stringify(payload), "utf8");
    const encrypted = Buffer.concat([cipher.update(encodedPayload), cipher.final()]);
    const authTag = cipher.getAuthTag();

    return {
      encrypted: true,
      algorithm: "aes-256-gcm",
      iv: iv.toString("base64"),
      tag: authTag.toString("base64"),
      payload: encrypted.toString("base64"),
    };
  }

  private calculateRetryDelay(attempt: number): number {
    const delay = RETRY_BASE_DELAY_MS * Math.pow(2, attempt - 1);
    return Math.min(delay, RETRY_MAX_DELAY_MS);
  }

  private getRateLimitDelayMs(webhook: WebhookRecord): number {
    const now = Date.now();
    const entry = this.rateWindow.get(webhook.id);

    if (!entry || now - entry.windowStart >= 60_000) {
      this.rateWindow.set(webhook.id, { windowStart: now, count: 1 });
      return 0;
    }

    if (entry.count >= webhook.rateLimitPerMinute) {
      return Math.max(1, 60_000 - (now - entry.windowStart));
    }

    entry.count += 1;
    this.rateWindow.set(webhook.id, entry);
    return 0;
  }

  private normalizeRateLimit(rateLimitPerMinute?: number): number {
    if (typeof rateLimitPerMinute === "undefined") {
      return 60;
    }
    if (!Number.isInteger(rateLimitPerMinute) || rateLimitPerMinute < 1) {
      throw new ValidationError("rateLimitPerMinute must be a positive integer");
    }
    return Math.min(rateLimitPerMinute, 6000);
  }

  private normalizeEvents(events: string[]): string[] {
    const normalized = Array.from(
      new Set(
        events
          .map((event) => event.trim().toLowerCase())
          .filter((event) => event.length > 0),
      ),
    );

    if (normalized.length === 0) {
      throw new ValidationError("At least one webhook event is required");
    }
    return normalized;
  }

  private normalizeAuthMethod(authMethod?: string): WebhookAuthMethod {
    if (typeof authMethod === "undefined") {
      return "SIGNATURE";
    }
    if (!WEBHOOK_AUTH_METHODS.includes(authMethod as WebhookAuthMethod)) {
      throw new ValidationError("Invalid authMethod");
    }
    return authMethod as WebhookAuthMethod;
  }

  private validateWebhookUrl(url: string): void {
    let parsed: URL;
    try {
      parsed = new URL(url);
    } catch {
      throw new ValidationError("Invalid webhook URL");
    }

    if (!["http:", "https:"].includes(parsed.protocol)) {
      throw new ValidationError("Webhook URL must use http or https");
    }
  }

  private async validateEndpointAccessibility(url: string): Promise<void> {
    const headers = { "user-agent": "stellovault-webhook-validator/1.0" };
    const attempts: Array<() => Promise<Response>> = [
      () =>
        this.fetchWithTimeout(url, {
          method: "HEAD",
          headers,
        }),
      () =>
        this.fetchWithTimeout(url, {
          method: "GET",
          headers,
        }),
    ];

    for (const attempt of attempts) {
      try {
        const response = await attempt();
        if (response.status < 400) {
          return;
        }
      } catch {
        // Continue to next check method.
      }
    }

    throw new ValidationError(
      "Webhook endpoint is not accessible (HEAD/GET check failed)",
    );
  }

  private async fetchWithTimeout(
    url: string,
    init: RequestInit,
    timeoutMs = 8000,
  ): Promise<Response> {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), timeoutMs);

    try {
      return await fetch(url, {
        ...init,
        signal: controller.signal,
      });
    } finally {
      clearTimeout(timeout);
    }
  }

  private async getOwnedWebhook(
    userId: string,
    webhookId: string,
  ): Promise<WebhookRecord> {
    const webhook = await this.prisma().webhook.findFirst({
      where: {
        id: webhookId,
        userId,
      },
    });

    if (!webhook) {
      throw new NotFoundError("Webhook not found");
    }

    return webhook as WebhookRecord;
  }

  private async sendDirectWebhook(
    webhookUrl: string,
    webhookSecret: string,
    eventType: string,
    payload: Record<string, unknown>,
  ): Promise<void> {
    const body = JSON.stringify(payload);
    const signature = signWebhookPayload(body, webhookSecret);
    const response = await this.fetchWithTimeout(webhookUrl, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-webhook-event": eventType,
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
}

export default new WebhookService();

import { Server, Api } from "@stellar/stellar-sdk";
import { env } from "../config/env";
import webhookService from "./webhook.service";
import logger from "../config/logger";
import { PrismaClient } from "../generated/prisma";
import notificationService from "./notification.service";

// ─────────────────────────────────────────────
// Event Types & Interfaces
// ─────────────────────────────────────────────

interface PaymentReceivedEventData {
  loanId: string;
  repaymentId: string;
  paymentSessionId?: string;
  checkoutUrl?: string;
  webhookUrl?: string | null;
  webhookSecret?: string | null;
  amount: string;
  outstandingAfter: string;
  paidAt: string;
  successUrl?: string;
  cancelUrl?: string;
}

interface MonitoringEvent {
  type: "PaymentReceived" | "ContractEvent" | "LedgerClose" | "Transaction" | "HealthAlert";
  payload: Record<string, unknown>;
  timestamp: Date;
  severity?: "info" | "warning" | "error" | "critical";
}

interface StellarTransaction {
  id: string;
  ledger: number;
  timestamp: Date;
  sourceAccount: string;
  operations: any[];
  memo?: string;
  fee: number;
  successful: boolean;
}

interface ContractEvent {
  contractId: string;
  eventType: string;
  topics: string[];
  data: Record<string, unknown>;
  transactionId: string;
  ledger: number;
  timestamp: Date;
}

interface HealthMetric {
  name: string;
  value: number;
  unit: string;
  timestamp: Date;
  threshold?: {
    warning: number;
    critical: number;
  };
}

interface AlertRule {
  id: string;
  name: string;
  eventType: string;
  condition: (event: MonitoringEvent) => boolean;
  severity: "info" | "warning" | "error" | "critical";
  channels: NotificationChannel[];
  cooldown: number; // milliseconds
  lastTriggered?: Date;
}

interface NotificationChannel {
  type: "email" | "slack" | "sms" | "webhook";
  config: Record<string, unknown>;
}

interface EventFilter {
  eventType?: string[];
  contractId?: string[];
  sourceAccount?: string[];
  minLedger?: number;
  maxLedger?: number;
  timeRange?: {
    start: Date;
    end: Date;
  };
}

// ─────────────────────────────────────────────
// Event Monitoring Service
// ─────────────────────────────────────────────

export class EventMonitoringService {
  private horizon: Server;
  private sorobanRpc: any;
  private prisma: PrismaClient;
  private isRunning: boolean = false;
  private pollInterval: NodeJS.Timeout | null = null;
  private eventQueue: MonitoringEvent[] = [];
  private alertRules: AlertRule[] = [];
  private healthMetrics: Map<string, HealthMetric[]> = new Map();
  private lastLedger: number = 0;
  private eventBuffer: Map<string, MonitoringEvent[]> = new Map();

  constructor() {
    this.horizon = new Server(env.stellar.horizonUrl);
    this.prisma = new PrismaClient();

    // Initialize Soroban RPC if configured
    if (env.stellar.sorobanRpcUrl) {
      this.sorobanRpc = new Server(env.stellar.sorobanRpcUrl);
    }

    this.initializeAlertRules();
    this.initializeHealthMetrics();
  }

  // ─────────────────────────────────────────────
  // Initialization
  // ─────────────────────────────────────────────

  private initializeAlertRules() {
    this.alertRules = [
      {
        id: "high-failure-rate",
        name: "High Transaction Failure Rate",
        eventType: "Transaction",
        condition: (event) => {
          const payload = event.payload as any;
          return !payload.successful && payload.amount > 1000;
        },
        severity: "error",
        channels: [{ type: "slack", config: { channel: "#alerts" } }],
        cooldown: 300000, // 5 minutes
      },
      {
        id: "contract-error",
        name: "Contract Execution Error",
        eventType: "ContractEvent",
        condition: (event) => {
          const payload = event.payload as ContractEvent;
          return payload.eventType.includes("error") || payload.eventType.includes("fail");
        },
        severity: "critical",
        channels: [
          { type: "slack", config: { channel: "#critical" } },
          { type: "email", config: { recipients: ["ops@stellovault.com"] } },
        ],
        cooldown: 60000, // 1 minute
      },
      {
        id: "ledger-lag",
        name: "Ledger Processing Lag",
        eventType: "LedgerClose",
        condition: (event) => {
          const payload = event.payload as any;
          const lag = Date.now() - new Date(payload.timestamp).getTime();
          return lag > 30000; // 30 seconds lag
        },
        severity: "warning",
        channels: [{ type: "slack", config: { channel: "#monitoring" } }],
        cooldown: 600000, // 10 minutes
      },
      {
        id: "high-latency",
        name: "High API Latency",
        eventType: "HealthAlert",
        condition: (event) => {
          const payload = event.payload as HealthMetric;
          return payload.name === "api_latency" && payload.value > 1000;
        },
        severity: "warning",
        channels: [{ type: "slack", config: { channel: "#performance" } }],
        cooldown: 300000,
      },
    ];
  }

  private initializeHealthMetrics() {
    this.healthMetrics.set("api_latency", []);
    this.healthMetrics.set("transaction_throughput", []);
    this.healthMetrics.set("ledger_close_time", []);
    this.healthMetrics.set("contract_execution_time", []);
    this.healthMetrics.set("database_query_time", []);
    this.healthMetrics.set("memory_usage", []);
    this.healthMetrics.set("cpu_usage", []);
  }

  // ─────────────────────────────────────────────
  // Main Monitoring Loop
  // ─────────────────────────────────────────────

  async start() {
    if (this.isRunning) {
      logger.warn("Event monitoring service is already running");
      return;
    }

    this.isRunning = true;
    logger.info("Starting event monitoring service");

    // Get the latest ledger to start from
    try {
      const latestLedger = await this.horizon.ledgers().order("desc").limit(1).call();
      this.lastLedger = latestLedger.records[0]?.sequence || 0;
      logger.info(`Starting from ledger ${this.lastLedger}`);
    } catch (error) {
      logger.error("Failed to get latest ledger:", error);
    }

    // Start polling for new ledgers
    this.pollInterval = setInterval(() => {
      this.pollEvents();
    }, 5000); // Poll every 5 seconds

    // Start health metrics collection
    setInterval(() => {
      this.collectHealthMetrics();
    }, 30000); // Every 30 seconds

    // Process event queue
    setInterval(() => {
      this.processEventQueue();
    }, 1000); // Every second
  }

  async stop() {
    this.isRunning = false;
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }
    logger.info("Event monitoring service stopped");
  }

  // ─────────────────────────────────────────────
  // Stellar Ledger Monitoring
  // ─────────────────────────────────────────────

  async pollEvents() {
    if (!this.isRunning) return;

    try {
      const startTime = Date.now();

      // Fetch new ledgers
      const ledgers = await this.horizon
        .ledgers()
        .order("asc")
        .limit(10)
        .cursor(this.lastLedger.toString())
        .call();

      for (const ledger of ledgers.records) {
        const ledgerSeq = parseInt(ledger.sequence);
        if (ledgerSeq <= this.lastLedger) continue;

        this.lastLedger = ledgerSeq;

        // Emit ledger close event
        const ledgerEvent: MonitoringEvent = {
          type: "LedgerClose",
          payload: {
            sequence: ledgerSeq,
            timestamp: ledger.closed_at,
            transactionCount: ledger.transaction_count,
            operationCount: ledger.operation_count,
            baseFee: ledger.base_fee_in_stroops,
            baseReserve: ledger.base_reserve_in_stroops,
          },
          timestamp: new Date(ledger.closed_at),
        };

        this.queueEvent(ledgerEvent);

        // Fetch transactions in this ledger
        await this.processLedgerTransactions(ledgerSeq);
      }

      // Track ledger close time metric
      const processingTime = Date.now() - startTime;
      this.recordHealthMetric("ledger_close_time", processingTime, "ms");

    } catch (error) {
      logger.error("Error polling events:", error);
      this.recordHealthMetric("api_latency", Date.now() - Date.now(), "ms");
    }
  }

  private async processLedgerTransactions(ledgerSeq: number) {
    try {
      const transactions = await this.horizon
        .transactions()
        .forLedger(ledgerSeq)
        .call();

      for (const tx of transactions.records) {
        const transactionEvent: MonitoringEvent = {
          type: "Transaction",
          payload: {
            id: tx.id,
            ledger: ledgerSeq,
            timestamp: tx.created_at,
            sourceAccount: tx.source_account,
            operations: tx.operations,
            memo: tx.memo,
            fee: tx.fee_charged,
            successful: tx.successful,
            resultXdr: tx.result_xdr,
          },
          timestamp: new Date(tx.created_at),
          severity: tx.successful ? "info" : "error",
        };

        this.queueEvent(transactionEvent);

        // Check for Soroban contract invocations
        if (tx.operations && Array.isArray(tx.operations)) {
          for (const op of tx.operations) {
            if (op.type === "invoke_host_function" || op.type === "extend_footprint_ttl") {
              await this.processSorobanOperation(tx.id, op);
            }
          }
        }
      }
    } catch (error) {
      logger.error(`Error processing transactions for ledger ${ledgerSeq}:`, error);
    }
  }

  // ─────────────────────────────────────────────
  // Soroban Contract Event Tracking
  // ─────────────────────────────────────────────

  private async processSorobanOperation(txId: string, operation: any) {
    try {
      if (!this.sorobanRpc) {
        logger.warn("Soroban RPC not configured, skipping contract event tracking");
        return;
      }

      // Get transaction details from Soroban RPC
      const txResult = await this.sorobanRpc.getTransaction(txId);

      if (txResult.status === "SUCCESS" && txResult.result?.events) {
        for (const event of txResult.result.events) {
          const contractEvent: MonitoringEvent = {
            type: "ContractEvent",
            payload: {
              contractId: event.contractId,
              eventType: event.type,
              topics: event.topics || [],
              data: event.data || {},
              transactionId: txId,
              ledger: txResult.ledger,
              timestamp: new Date(),
            },
            timestamp: new Date(),
            severity: "info",
          };

          this.queueEvent(contractEvent);
        }
      }
    } catch (error) {
      logger.error(`Error processing Soroban operation for tx ${txId}:`, error);
    }
  }

  async getContractEvents(
    contractId: string,
    filter?: EventFilter,
  ): Promise<ContractEvent[]> {
    try {
      if (!this.sorobanRpc) {
        throw new Error("Soroban RPC not configured");
      }

      const events = await this.sorobanRpc.getEvents({
        filters: [{ type: "contract", contractIds: [contractId] }],
        ...filter,
      });

      return events.map((e: any) => ({
        contractId: e.contractId,
        eventType: e.type,
        topics: e.topics || [],
        data: e.data || {},
        transactionId: e.txHash,
        ledger: e.ledger,
        timestamp: new Date(),
      }));
    } catch (error) {
      logger.error(`Error getting contract events for ${contractId}:`, error);
      return [];
    }
  }

  // ─────────────────────────────────────────────
  // Platform Health Metrics
  // ─────────────────────────────────────────────

  async collectHealthMetrics() {
    const startTime = Date.now();

    try {
      // API Latency
      const apiStart = Date.now();
      await this.horizon.ledgers().limit(1).call();
      const apiLatency = Date.now() - apiStart;
      this.recordHealthMetric("api_latency", apiLatency, "ms", {
        warning: 500,
        critical: 2000,
      });

      // Memory Usage
      const memoryUsage = process.memoryUsage();
      this.recordHealthMetric("memory_usage", memoryUsage.heapUsed / 1024 / 1024, "MB", {
        warning: 500,
        critical: 1000,
      });

      // CPU Usage (approximate)
      const cpuUsage = process.cpuUsage();
      this.recordHealthMetric("cpu_usage", cpuUsage.user / 1000000, "seconds");

      // Database Query Time
      const dbStart = Date.now();
      await this.prisma.$queryRaw`SELECT 1`;
      const dbTime = Date.now() - dbStart;
      this.recordHealthMetric("database_query_time", dbTime, "ms", {
        warning: 100,
        critical: 500,
      });

      // Transaction Throughput (transactions per minute)
      const throughput = this.calculateTransactionThroughput();
      this.recordHealthMetric("transaction_throughput", throughput, "tx/min");

      // Check for health alerts
      await this.checkHealthAlerts();

    } catch (error) {
      logger.error("Error collecting health metrics:", error);
    }
  }

  private recordHealthMetric(
    name: string,
    value: number,
    unit: string,
    threshold?: { warning: number; critical: number },
  ) {
    const metric: HealthMetric = {
      name,
      value,
      unit,
      timestamp: new Date(),
      threshold,
    };

    const metrics = this.healthMetrics.get(name) || [];
    metrics.push(metric);

    // Keep only last 1000 data points
    if (metrics.length > 1000) {
      metrics.shift();
    }

    this.healthMetrics.set(name, metrics);

    // Log to database for historical analysis
    this.persistHealthMetric(metric);
  }

  private async persistHealthMetric(metric: HealthMetric) {
    try {
      await this.prisma.healthMetric.create({
        data: {
          name: metric.name,
          value: metric.value,
          unit: metric.unit,
          timestamp: metric.timestamp,
        },
      });
    } catch (error) {
      logger.error("Error persisting health metric:", error);
    }
  }

  private calculateTransactionThroughput(): number {
    const now = Date.now();
    const oneMinuteAgo = now - 60000;

    const recentEvents = this.eventQueue.filter(
      (e) => e.type === "Transaction" && e.timestamp.getTime() > oneMinuteAgo,
    );

    return recentEvents.length;
  }

  private async checkHealthAlerts() {
    for (const [metricName, metrics] of this.healthMetrics.entries()) {
      if (metrics.length === 0) continue;

      const latest = metrics[metrics.length - 1];
      if (!latest.threshold) continue;

      let severity: "warning" | "critical" | null = null;
      if (latest.value >= latest.threshold.critical) {
        severity = "critical";
      } else if (latest.value >= latest.threshold.warning) {
        severity = "warning";
      }

      if (severity) {
        const alertEvent: MonitoringEvent = {
          type: "HealthAlert",
          payload: latest,
          timestamp: new Date(),
          severity,
        };

        this.queueEvent(alertEvent);
      }
    }
  }

  // ─────────────────────────────────────────────
  // Event Processing & Routing
  // ─────────────────────────────────────────────

  private queueEvent(event: MonitoringEvent) {
    this.eventQueue.push(event);

    // Log event for audit trail
    this.logEvent(event);

    // Check alert rules
    this.checkAlertRules(event);
  }

  private async processEventQueue() {
    if (this.eventQueue.length === 0) return;

    const events = [...this.eventQueue];
    this.eventQueue = [];

    for (const event of events) {
      await this.processEvent(event);
    }
  }

  async processEvent(event: MonitoringEvent) {
    try {
      switch (event.type) {
        case "PaymentReceived":
          await this.handlePaymentReceived(event.payload as unknown as PaymentReceivedEventData);
          break;

        case "ContractEvent":
          await this.handleContractEvent(event.payload as ContractEvent);
          break;

        case "Transaction":
          await this.handleTransaction(event.payload as StellarTransaction);
          break;

        case "HealthAlert":
          await this.handleHealthAlert(event);
          break;

        default:
          logger.debug(`Unknown event type: ${event.type}`);
      }
    } catch (error) {
      logger.error(`Error processing event ${event.type}:`, error);
    }
  }

  private async handlePaymentReceived(event: PaymentReceivedEventData) {
    if (!event.webhookUrl) {
      return;
    }

    const webhookSecret = event.webhookSecret?.trim();
    if (!webhookSecret) {
      logger.warn(
        `Skipping payment webhook for loan ${event.loanId}: no webhook secret configured`,
      );
      return;
    }

    await webhookService.enqueuePaymentReceivedWebhook({
      loanId: event.loanId,
      repaymentId: event.repaymentId,
      paymentSessionId: event.paymentSessionId ?? event.repaymentId,
      webhookUrl: event.webhookUrl,
      webhookSecret,
      payload: {
        event: "payment.received",
        loanId: event.loanId,
        repaymentId: event.repaymentId,
        paymentSessionId: event.paymentSessionId ?? null,
        checkoutUrl: event.checkoutUrl ?? null,
        amount: event.amount,
        outstandingAfter: event.outstandingAfter,
        paidAt: event.paidAt,
        successUrl: event.successUrl ?? null,
        cancelUrl: event.cancelUrl ?? null,
      },
    });
  }

  private async handleContractEvent(event: ContractEvent) {
    // Store contract event in database
    try {
      await this.prisma.contractEvent.create({
        data: {
          contractId: event.contractId,
          eventType: event.eventType,
          topics: event.topics,
          data: event.data as any,
          transactionId: event.transactionId,
          ledger: event.ledger,
          timestamp: event.timestamp,
        },
      });
    } catch (error) {
      logger.error("Error storing contract event:", error);
    }

    // Route to interested parties based on contract type
    await this.routeContractEvent(event);
  }

  private async handleTransaction(event: StellarTransaction) {
    // Store transaction in database
    try {
      await this.prisma.stellarTransaction.create({
        data: {
          id: event.id,
          ledger: event.ledger,
          timestamp: event.timestamp,
          sourceAccount: event.sourceAccount,
          operations: event.operations as any,
          memo: event.memo,
          fee: event.fee,
          successful: event.successful,
        },
      });
    } catch (error) {
      logger.error("Error storing transaction:", error);
    }
  }

  private async handleHealthAlert(event: MonitoringEvent) {
    const metric = event.payload as HealthMetric;
    logger.warn(`Health Alert: ${metric.name} = ${metric.value}${metric.unit}`);

    // Send notifications based on severity
    await this.sendAlertNotifications(event);
  }

  private async routeContractEvent(event: ContractEvent) {
    // Implement routing logic based on contract type
    // e.g., escrow events go to escrow service, loan events to loan service
    if (event.contractId.includes("escrow")) {
      // Route to escrow service
    } else if (event.contractId.includes("loan")) {
      // Route to loan service
    } else if (event.contractId.includes("oracle")) {
      // Route to oracle service
    }
  }

  // ─────────────────────────────────────────────
  // Alerting System
  // ─────────────────────────────────────────────

  private checkAlertRules(event: MonitoringEvent) {
    for (const rule of this.alertRules) {
      if (rule.eventType !== event.type) continue;

      // Check cooldown
      if (rule.lastTriggered) {
        const timeSinceLastTrigger = Date.now() - rule.lastTriggered.getTime();
        if (timeSinceLastTrigger < rule.cooldown) continue;
      }

      // Check condition
      if (rule.condition(event)) {
        rule.lastTriggered = new Date();
        this.triggerAlert(rule, event);
      }
    }
  }

  private async triggerAlert(rule: AlertRule, event: MonitoringEvent) {
    logger.info(`Alert triggered: ${rule.name}`, { event });

    for (const channel of rule.channels) {
      await this.sendNotification(channel, rule, event);
    }
  }

  private async sendAlertNotifications(event: MonitoringEvent) {
    const channels: NotificationChannel[] = [
      { type: "slack", config: { channel: "#alerts" } },
    ];

    if (event.severity === "critical") {
      channels.push({ type: "email", config: { recipients: ["ops@stellovault.com"] } });
    }

    for (const channel of channels) {
      await this.sendNotification(channel, null, event);
    }
  }

  private async sendNotification(
    channel: NotificationChannel,
    rule: AlertRule | null,
    event: MonitoringEvent,
  ) {
    try {
      switch (channel.type) {
        case "slack":
          await this.sendSlackNotification(channel.config, rule, event);
          break;
        case "email":
          await this.sendEmailNotification(channel.config, rule, event);
          break;
        case "sms":
          await this.sendSmsNotification(channel.config, rule, event);
          break;
        case "webhook":
          await this.sendWebhookNotification(channel.config, rule, event);
          break;
      }
    } catch (error) {
      logger.error(`Error sending ${channel.type} notification:`, error);
    }
  }

  private async sendSlackNotification(
    config: Record<string, unknown>,
    rule: AlertRule | null,
    event: MonitoringEvent,
  ) {
    const notificationPayload: NotificationPayload = {
      alertId: rule?.id || "manual",
      ruleName: rule?.name || "Manual Alert",
      eventType: event.type,
      severity: event.severity || "info",
      payload: event.payload,
      timestamp: event.timestamp,
    };

    await notificationService.sendSlack(
      config as SlackConfig,
      notificationPayload,
    );
  }

  private async sendEmailNotification(
    config: Record<string, unknown>,
    rule: AlertRule | null,
    event: MonitoringEvent,
  ) {
    const notificationPayload: NotificationPayload = {
      alertId: rule?.id || "manual",
      ruleName: rule?.name || "Manual Alert",
      eventType: event.type,
      severity: event.severity || "info",
      payload: event.payload,
      timestamp: event.timestamp,
    };

    await notificationService.sendEmail(
      config as EmailConfig,
      notificationPayload,
    );
  }

  private async sendSmsNotification(
    config: Record<string, unknown>,
    rule: AlertRule | null,
    event: MonitoringEvent,
  ) {
    const notificationPayload: NotificationPayload = {
      alertId: rule?.id || "manual",
      ruleName: rule?.name || "Manual Alert",
      eventType: event.type,
      severity: event.severity || "info",
      payload: event.payload,
      timestamp: event.timestamp,
    };

    await notificationService.sendSms(
      config as SmsConfig,
      notificationPayload,
    );
  }

  private async sendWebhookNotification(
    config: Record<string, unknown>,
    rule: AlertRule | null,
    event: MonitoringEvent,
  ) {
    const notificationPayload: NotificationPayload = {
      alertId: rule?.id || "manual",
      ruleName: rule?.name || "Manual Alert",
      eventType: event.type,
      severity: event.severity || "info",
      payload: event.payload,
      timestamp: event.timestamp,
    };

    await notificationService.sendWebhook(
      config as WebhookConfig,
      notificationPayload,
    );
  }

  // ─────────────────────────────────────────────
  // Event Logging & Audit Trail
  // ─────────────────────────────────────────────

  private async logEvent(event: MonitoringEvent) {
    try {
      await this.prisma.eventLog.create({
        data: {
          type: event.type,
          payload: event.payload as any,
          timestamp: event.timestamp,
          severity: event.severity || "info",
        },
      });
    } catch (error) {
      logger.error("Error logging event:", error);
    }
  }

  async getEventLogs(filter?: EventFilter): Promise<MonitoringEvent[]> {
    const where: any = {};

    if (filter?.eventType) {
      where.type = { in: filter.eventType };
    }

    if (filter?.timeRange) {
      where.timestamp = {
        gte: filter.timeRange.start,
        lte: filter.timeRange.end,
      };
    }

    const logs = await this.prisma.eventLog.findMany({
      where,
      orderBy: { timestamp: "desc" },
      take: 1000,
    });

    return logs.map((log) => ({
      type: log.type,
      payload: log.payload as Record<string, unknown>,
      timestamp: log.timestamp,
      severity: log.severity as any,
    }));
  }

  // ─────────────────────────────────────────────
  // Query & Analytics Methods
  // ─────────────────────────────────────────────

  async getHealthMetrics(metricName?: string, timeRange?: { start: Date; end: Date }) {
    const where: any = {};

    if (metricName) {
      where.name = metricName;
    }

    if (timeRange) {
      where.timestamp = {
        gte: timeRange.start,
        lte: timeRange.end,
      };
    }

    return this.prisma.healthMetric.findMany({
      where,
      orderBy: { timestamp: "desc" },
      take: 1000,
    });
  }

  async getTransactionStats(timeRange: { start: Date; end: Date }) {
    const transactions = await this.prisma.stellarTransaction.findMany({
      where: {
        timestamp: {
          gte: timeRange.start,
          lte: timeRange.end,
        },
      },
    });

    const total = transactions.length;
    const successful = transactions.filter((t) => t.successful).length;
    const failed = total - successful;
    const totalFee = transactions.reduce((sum, t) => sum + t.fee, 0);

    return {
      total,
      successful,
      failed,
      successRate: total > 0 ? (successful / total) * 100 : 0,
      totalFee,
      averageFee: total > 0 ? totalFee / total : 0,
    };
  }

  async getContractEventStats(contractId: string, timeRange?: { start: Date; end: Date }) {
    const where: any = { contractId };

    if (timeRange) {
      where.timestamp = {
        gte: timeRange.start,
        lte: timeRange.end,
      };
    }

    const events = await this.prisma.contractEvent.findMany({
      where,
      orderBy: { timestamp: "desc" },
      take: 1000,
    });

    // Group by event type
    const byType = events.reduce((acc, event) => {
      acc[event.eventType] = (acc[event.eventType] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    return {
      total: events.length,
      byType,
      uniqueTransactions: new Set(events.map((e) => e.transactionId)).size,
    };
  }

  // ─────────────────────────────────────────────
  // Dashboard Data
  // ─────────────────────────────────────────────

  async getDashboardData() {
    const now = new Date();
    const oneHourAgo = new Date(now.getTime() - 3600000);
    const oneDayAgo = new Date(now.getTime() - 86400000);

    const [
      recentEvents,
      healthMetrics,
      transactionStats,
      ledgerInfo,
    ] = await Promise.all([
      this.getEventLogs({ timeRange: { start: oneHourAgo, end: now } }),
      this.getHealthMetrics(undefined, { start: oneHourAgo, end: now }),
      this.getTransactionStats({ start: oneDayAgo, end: now }),
      this.horizon.ledgers().order("desc").limit(1).call(),
    ]);

    return {
      recentEvents: recentEvents.slice(0, 50),
      healthMetrics: this.aggregateHealthMetrics(healthMetrics),
      transactionStats,
      ledgerInfo: ledgerInfo.records[0] || null,
      isRunning: this.isRunning,
      lastLedger: this.lastLedger,
      eventQueueSize: this.eventQueue.length,
    };
  }

  private aggregateHealthMetrics(metrics: any[]) {
    const aggregated: Record<string, any> = {};

    for (const metric of metrics) {
      if (!aggregated[metric.name]) {
        aggregated[metric.name] = {
          name: metric.name,
          current: metric.value,
          unit: metric.unit,
          min: metric.value,
          max: metric.value,
          avg: metric.value,
          count: 1,
        };
      } else {
        const agg = aggregated[metric.name];
        agg.current = metric.value;
        agg.min = Math.min(agg.min, metric.value);
        agg.max = Math.max(agg.max, metric.value);
        agg.avg = (agg.avg * agg.count + metric.value) / (agg.count + 1);
        agg.count += 1;
      }
    }

    return Object.values(aggregated);
  }
}

export default new EventMonitoringService();

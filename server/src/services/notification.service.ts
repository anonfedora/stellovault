import { PrismaClient } from "../generated/prisma";
import logger from "../config/logger";
import { env } from "../config/env";

// ─────────────────────────────────────────────
// Notification Types & Interfaces
// ─────────────────────────────────────────────

interface NotificationPayload {
  alertId: string;
  ruleName: string;
  eventType: string;
  severity: "info" | "warning" | "error" | "critical";
  payload: Record<string, unknown>;
  timestamp: Date;
}

interface EmailConfig {
  recipients: string[];
  subject?: string;
  template?: string;
}

interface SlackConfig {
  channel: string;
  username?: string;
  iconEmoji?: string;
}

interface SmsConfig {
  recipients: string[];
  message?: string;
}

interface WebhookConfig {
  url: string;
  headers?: Record<string, string>;
  method?: "POST" | "PUT" | "PATCH";
}

// ─────────────────────────────────────────────
// Notification Service
// ─────────────────────────────────────────────

export class NotificationService {
  private prisma: PrismaClient;

  constructor() {
    this.prisma = new PrismaClient();
  }

  // ─────────────────────────────────────────────
  // Email Notifications
  // ─────────────────────────────────────────────

  async sendEmail(
    config: EmailConfig,
    payload: NotificationPayload,
  ): Promise<{ success: boolean; error?: string }> {
    try {
      const subject = config.subject || this.generateEmailSubject(payload);
      const body = this.generateEmailBody(payload);

      // TODO: Integrate with email service (e.g., SendGrid, AWS SES, Nodemailer)
      // For now, log the email that would be sent
      logger.info("Email notification prepared", {
        recipients: config.recipients,
        subject,
        bodyLength: body.length,
      });

      // Store notification record
      await this.prisma.notification.create({
        data: {
          alertId: payload.alertId,
          channel: "EMAIL",
          status: "SENT",
          recipient: config.recipients.join(","),
          payload: { ...config, subject, body },
          sentAt: new Date(),
        },
      });

      return { success: true };
    } catch (error) {
      logger.error("Failed to send email notification:", error);
      return { success: false, error: (error as Error).message };
    }
  }

  private generateEmailSubject(payload: NotificationPayload): string {
    const severityEmoji = {
      info: "ℹ️",
      warning: "⚠️",
      error: "❌",
      critical: "🚨",
    };

    return `${severityEmoji[payload.severity]} [${payload.severity.toUpperCase()}] ${payload.ruleName}`;
  }

  private generateEmailBody(payload: NotificationPayload): string {
    return `
Alert Details:
-------------
Rule: ${payload.ruleName}
Event Type: ${payload.eventType}
Severity: ${payload.severity}
Timestamp: ${payload.timestamp.toISOString()}

Payload:
--------
${JSON.stringify(payload.payload, null, 2)}

---
This is an automated alert from StelloVault Monitoring System.
    `.trim();
  }

  // ─────────────────────────────────────────────
  // Slack Notifications
  // ─────────────────────────────────────────────

  async sendSlack(
    config: SlackConfig,
    payload: NotificationPayload,
  ): Promise<{ success: boolean; error?: string }> {
    try {
      const webhookUrl = env.slack?.webhookUrl;
      if (!webhookUrl) {
        logger.warn("Slack webhook URL not configured");
        return { success: false, error: "Slack webhook URL not configured" };
      }

      const slackPayload = this.generateSlackPayload(config, payload);

      const response = await fetch(webhookUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(slackPayload),
      });

      if (!response.ok) {
        throw new Error(`Slack API returned ${response.status}`);
      }

      // Store notification record
      await this.prisma.notification.create({
        data: {
          alertId: payload.alertId,
          channel: "SLACK",
          status: "SENT",
          recipient: config.channel,
          payload: slackPayload,
          sentAt: new Date(),
        },
      });

      return { success: true };
    } catch (error) {
      logger.error("Failed to send Slack notification:", error);

      // Store failed notification
      await this.prisma.notification.create({
        data: {
          alertId: payload.alertId,
          channel: "SLACK",
          status: "FAILED",
          recipient: config.channel,
          payload: config,
          failedAt: new Date(),
          errorMessage: (error as Error).message,
        },
      });

      return { success: false, error: (error as Error).message };
    }
  }

  private generateSlackPayload(
    config: SlackConfig,
    payload: NotificationPayload,
  ): Record<string, unknown> {
    const color = {
      info: "#36a64f",
      warning: "#ffaa00",
      error: "#ff0000",
      critical: "#ff0000",
    };

    return {
      channel: config.channel,
      username: config.username || "StelloVault Monitor",
      icon_emoji: config.iconEmoji || ":robot_face:",
      attachments: [
        {
          color: color[payload.severity],
          title: payload.ruleName,
          fields: [
            {
              title: "Event Type",
              value: payload.eventType,
              short: true,
            },
            {
              title: "Severity",
              value: payload.severity.toUpperCase(),
              short: true,
            },
            {
              title: "Timestamp",
              value: payload.timestamp.toISOString(),
              short: true,
            },
          ],
          text: `\`\`\`${JSON.stringify(payload.payload, null, 2)}\`\`\``,
          footer: "StelloVault Monitoring",
          ts: Math.floor(payload.timestamp.getTime() / 1000),
        },
      ],
    };
  }

  // ─────────────────────────────────────────────
  // SMS Notifications
  // ─────────────────────────────────────────────

  async sendSms(
    config: SmsConfig,
    payload: NotificationPayload,
  ): Promise<{ success: boolean; error?: string }> {
    try {
      const message = config.message || this.generateSmsMessage(payload);

      // TODO: Integrate with SMS service (e.g., Twilio, AWS SNS)
      // For now, log the SMS that would be sent
      logger.info("SMS notification prepared", {
        recipients: config.recipients,
        messageLength: message.length,
      });

      // Store notification record
      await this.prisma.notification.create({
        data: {
          alertId: payload.alertId,
          channel: "SMS",
          status: "SENT",
          recipient: config.recipients.join(","),
          payload: { message },
          sentAt: new Date(),
        },
      });

      return { success: true };
    } catch (error) {
      logger.error("Failed to send SMS notification:", error);
      return { success: false, error: (error as Error).message };
    }
  }

  private generateSmsMessage(payload: NotificationPayload): string {
    const severityMap = {
      info: "INFO",
      warning: "WARN",
      error: "ERR",
      critical: "CRIT",
    };

    return `[${severityMap[payload.severity]}] ${payload.ruleName}: ${payload.eventType} at ${payload.timestamp.toISOString()}`;
  }

  // ─────────────────────────────────────────────
  // Webhook Notifications
  // ─────────────────────────────────────────────

  async sendWebhook(
    config: WebhookConfig,
    payload: NotificationPayload,
  ): Promise<{ success: boolean; error?: string }> {
    try {
      const webhookPayload = {
        alert: {
          id: payload.alertId,
          ruleName: payload.ruleName,
          eventType: payload.eventType,
          severity: payload.severity,
          timestamp: payload.timestamp.toISOString(),
        },
        data: payload.payload,
      };

      const response = await fetch(config.url, {
        method: config.method || "POST",
        headers: {
          "Content-Type": "application/json",
          ...config.headers,
        },
        body: JSON.stringify(webhookPayload),
      });

      if (!response.ok) {
        throw new Error(`Webhook returned ${response.status}`);
      }

      // Store notification record
      await this.prisma.notification.create({
        data: {
          alertId: payload.alertId,
          channel: "WEBHOOK",
          status: "SENT",
          recipient: config.url,
          payload: webhookPayload,
          sentAt: new Date(),
        },
      });

      return { success: true };
    } catch (error) {
      logger.error("Failed to send webhook notification:", error);

      // Store failed notification
      await this.prisma.notification.create({
        data: {
          alertId: payload.alertId,
          channel: "WEBHOOK",
          status: "FAILED",
          recipient: config.url,
          payload: config,
          failedAt: new Date(),
          errorMessage: (error as Error).message,
        },
      });

      return { success: false, error: (error as Error).message };
    }
  }

  // ─────────────────────────────────────────────
  // Retry Failed Notifications
  // ─────────────────────────────────────────────

  async retryFailedNotifications(maxRetries: number = 3): Promise<void> {
    const failedNotifications = await this.prisma.notification.findMany({
      where: {
        status: "FAILED",
        retryCount: { lt: maxRetries },
      },
      take: 100,
    });

    for (const notification of failedNotifications) {
      try {
        await this.prisma.notification.update({
          where: { id: notification.id },
          data: {
            status: "RETRYING",
            retryCount: { increment: 1 },
          },
        });

        // Retry based on channel
        let success = false;
        const payload = notification.payload as any;

        switch (notification.channel) {
          case "EMAIL":
            const emailResult = await this.sendEmail(
              { recipients: notification.recipient.split(",") },
              payload,
            );
            success = emailResult.success;
            break;

          case "SLACK":
            const slackResult = await this.sendSlack(
              { channel: notification.recipient },
              payload,
            );
            success = slackResult.success;
            break;

          case "SMS":
            const smsResult = await this.sendSms(
              { recipients: notification.recipient.split(",") },
              payload,
            );
            success = smsResult.success;
            break;

          case "WEBHOOK":
            const webhookResult = await this.sendWebhook(
              { url: notification.recipient },
              payload,
            );
            success = webhookResult.success;
            break;
        }

        if (success) {
          await this.prisma.notification.update({
            where: { id: notification.id },
            data: { status: "SENT", sentAt: new Date() },
          });
        }
      } catch (error) {
        logger.error(`Failed to retry notification ${notification.id}:`, error);
      }
    }
  }

  // ─────────────────────────────────────────────
  // Notification History & Analytics
  // ─────────────────────────────────────────────

  async getNotificationHistory(
    filter?: {
      channel?: "EMAIL" | "SLACK" | "SMS" | "WEBHOOK";
      status?: "PENDING" | "SENT" | "FAILED" | "RETRYING";
      startTime?: Date;
      endTime?: Date;
    },
  ) {
    const where: any = {};

    if (filter?.channel) {
      where.channel = filter.channel;
    }

    if (filter?.status) {
      where.status = filter.status;
    }

    if (filter?.startTime || filter?.endTime) {
      where.createdAt = {};
      if (filter.startTime) where.createdAt.gte = filter.startTime;
      if (filter.endTime) where.createdAt.lte = filter.endTime;
    }

    return this.prisma.notification.findMany({
      where,
      orderBy: { createdAt: "desc" },
      take: 1000,
    });
  }

  async getNotificationStats(timeRange: { start: Date; end: Date }) {
    const notifications = await this.prisma.notification.findMany({
      where: {
        createdAt: {
          gte: timeRange.start,
          lte: timeRange.end,
        },
      },
    });

    const byChannel = notifications.reduce((acc, n) => {
      acc[n.channel] = (acc[n.channel] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const byStatus = notifications.reduce((acc, n) => {
      acc[n.status] = (acc[n.status] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const total = notifications.length;
    const sent = notifications.filter((n) => n.status === "SENT").length;
    const failed = notifications.filter((n) => n.status === "FAILED").length;

    return {
      total,
      sent,
      failed,
      successRate: total > 0 ? (sent / total) * 100 : 0,
      byChannel,
      byStatus,
    };
  }
}

export default new NotificationService();

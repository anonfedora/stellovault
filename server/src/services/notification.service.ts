import sgMail from "@sendgrid/mail";
import { EventEmitter } from "events";
import websocketService from "./websocket.service";
import { prisma } from "./database.service";

const SENDGRID_API_KEY = process.env.SENDGRID_API_KEY || "";
const FROM_EMAIL = process.env.FROM_EMAIL || "noreply@stellovault.com";
const NOTIFICATION_TIMEOUT_MS = 60000; // 60 seconds

if (SENDGRID_API_KEY) {
    sgMail.setApiKey(SENDGRID_API_KEY);
}

export enum NotificationType {
    LOAN_APPROVED = "LOAN_APPROVED",
    LOAN_REPAYMENT_DUE = "LOAN_REPAYMENT_DUE",
    ESCROW_EXPIRING_SOON = "ESCROW_EXPIRING_SOON",
    ESCROW_RELEASED = "ESCROW_RELEASED",
    SECURITY_ALERT = "SECURITY_ALERT",
    COLLATERAL_LOCKED = "COLLATERAL_LOCKED",
    ORACLE_CONFIRMATION = "ORACLE_CONFIRMATION",
    GOVERNANCE_PROPOSAL = "GOVERNANCE_PROPOSAL",
}

export enum NotificationChannel {
    EMAIL = "EMAIL",
    WEBSOCKET = "WEBSOCKET",
    BOTH = "BOTH",
}

interface NotificationPayload {
    userId: string;
    type: NotificationType;
    channel: NotificationChannel;
    data: Record<string, any>;
    priority?: "low" | "normal" | "high";
}

interface EmailTemplate {
    subject: string;
    html: string;
    text: string;
}

export class NotificationService extends EventEmitter {
    private notificationQueue: NotificationPayload[] = [];
    private processing = false;

    constructor() {
        super();
        this.startProcessor();
    }

    /**
     * Send a notification to a user
     */
    async sendNotification(payload: NotificationPayload): Promise<void> {
        const { userId, type, channel, data, priority = "normal" } = payload;

        // Check user preferences
        const preferences = await this.getUserPreferences(userId);
        if (!this.shouldSendNotification(type, preferences)) {
            console.log(`User ${userId} has opted out of ${type} notifications`);
            return;
        }

        // Add to queue for async processing
        this.notificationQueue.push(payload);

        // Emit event for monitoring
        this.emit("notification:queued", { userId, type, channel });

        // Process queue
        this.processQueue();
    }

    private async processQueue(): Promise<void> {
        if (this.processing || this.notificationQueue.length === 0) {
            return;
        }

        this.processing = true;

        while (this.notificationQueue.length > 0) {
            const payload = this.notificationQueue.shift();
            if (!payload) continue;

            try {
                await this.deliverNotification(payload);
            } catch (error) {
                console.error(`Failed to deliver notification:`, error);
                this.emit("notification:failed", { payload, error });
            }
        }

        this.processing = false;
    }

    private async deliverNotification(payload: NotificationPayload): Promise<void> {
        const { userId, type, channel, data } = payload;
        const startTime = Date.now();

        const promises: Promise<any>[] = [];

        // Send via WebSocket (real-time)
        if (channel === NotificationChannel.WEBSOCKET || channel === NotificationChannel.BOTH) {
            promises.push(this.sendWebSocketNotification(userId, type, data));
        }

        // Send via Email (asynchronous)
        if (channel === NotificationChannel.EMAIL || channel === NotificationChannel.BOTH) {
            promises.push(this.sendEmailNotification(userId, type, data));
        }

        await Promise.allSettled(promises);

        const duration = Date.now() - startTime;
        console.log(`Notification ${type} delivered to user ${userId} in ${duration}ms`);

        // Check if within 60s SLA
        if (duration > NOTIFICATION_TIMEOUT_MS) {
            console.warn(`⚠️ Notification ${type} exceeded 60s SLA: ${duration}ms`);
        }

        this.emit("notification:delivered", { userId, type, duration });
    }

    private async sendWebSocketNotification(
        userId: string,
        type: NotificationType,
        data: Record<string, any>
    ): Promise<void> {
        // Send real-time notification via WebSocket
        websocketService.broadcast({
            type: "NOTIFICATION",
            notificationType: type,
            userId,
            data,
            timestamp: new Date().toISOString(),
        });
    }

    private async sendEmailNotification(
        userId: string,
        type: NotificationType,
        data: Record<string, any>
    ): Promise<void> {
        if (!SENDGRID_API_KEY) {
            console.warn("SendGrid API key not configured, skipping email");
            return;
        }

        // Get user email
        const user = await prisma.user.findUnique({
            where: { id: userId },
            select: { stellarAddress: true, name: true },
        });

        if (!user) {
            throw new Error(`User ${userId} not found`);
        }

        // For demo purposes, use stellar address as email
        // In production, you'd have a separate email field
        const userEmail = `${user.stellarAddress.substring(0, 8)}@example.com`;

        const template = this.getEmailTemplate(type, data, user.name || "User");

        const msg = {
            to: userEmail,
            from: FROM_EMAIL,
            subject: template.subject,
            text: template.text,
            html: template.html,
        };

        try {
            await sgMail.send(msg);
            console.log(`✓ Email sent to ${userEmail} for ${type}`);
        } catch (error: any) {
            console.error(`✗ Failed to send email:`, error.response?.body || error.message);
            throw error;
        }
    }

    private getEmailTemplate(
        type: NotificationType,
        data: Record<string, any>,
        userName: string
    ): EmailTemplate {
        switch (type) {
            case NotificationType.LOAN_APPROVED:
                return {
                    subject: "🎉 Your Loan Has Been Approved!",
                    html: `
                        <h2>Congratulations, ${userName}!</h2>
                        <p>Your loan application has been approved.</p>
                        <ul>
                            <li><strong>Loan ID:</strong> ${data.loanId}</li>
                            <li><strong>Amount:</strong> ${data.amount} ${data.assetCode}</li>
                            <li><strong>Interest Rate:</strong> ${data.interestRate}%</li>
                            <li><strong>Due Date:</strong> ${data.dueDate}</li>
                        </ul>
                        <p>The funds will be transferred to your account shortly.</p>
                        <p><a href="${data.dashboardUrl}">View Loan Details</a></p>
                    `,
                    text: `Congratulations ${userName}! Your loan (${data.loanId}) for ${data.amount} ${data.assetCode} has been approved.`,
                };

            case NotificationType.LOAN_REPAYMENT_DUE:
                return {
                    subject: "⏰ Loan Repayment Due Soon",
                    html: `
                        <h2>Repayment Reminder</h2>
                        <p>Hi ${userName},</p>
                        <p>Your loan repayment is due soon.</p>
                        <ul>
                            <li><strong>Loan ID:</strong> ${data.loanId}</li>
                            <li><strong>Amount Due:</strong> ${data.amountDue} ${data.assetCode}</li>
                            <li><strong>Due Date:</strong> ${data.dueDate}</li>
                            <li><strong>Days Remaining:</strong> ${data.daysRemaining}</li>
                        </ul>
                        <p>Please ensure you have sufficient funds to avoid late fees.</p>
                        <p><a href="${data.repaymentUrl}">Make Payment</a></p>
                    `,
                    text: `Hi ${userName}, your loan repayment of ${data.amountDue} ${data.assetCode} is due on ${data.dueDate}.`,
                };

            case NotificationType.ESCROW_EXPIRING_SOON:
                return {
                    subject: "⚠️ Escrow Expiring Soon",
                    html: `
                        <h2>Escrow Expiration Notice</h2>
                        <p>Hi ${userName},</p>
                        <p>Your escrow is expiring soon.</p>
                        <ul>
                            <li><strong>Escrow ID:</strong> ${data.escrowId}</li>
                            <li><strong>Amount:</strong> ${data.amount} ${data.assetCode}</li>
                            <li><strong>Expires At:</strong> ${data.expiresAt}</li>
                            <li><strong>Hours Remaining:</strong> ${data.hoursRemaining}</li>
                        </ul>
                        <p>Please take action before expiration to avoid automatic refund.</p>
                        <p><a href="${data.escrowUrl}">View Escrow</a></p>
                    `,
                    text: `Hi ${userName}, your escrow (${data.escrowId}) expires on ${data.expiresAt}.`,
                };

            case NotificationType.ESCROW_RELEASED:
                return {
                    subject: "✅ Escrow Funds Released",
                    html: `
                        <h2>Funds Released</h2>
                        <p>Hi ${userName},</p>
                        <p>The escrow funds have been successfully released.</p>
                        <ul>
                            <li><strong>Escrow ID:</strong> ${data.escrowId}</li>
                            <li><strong>Amount:</strong> ${data.amount} ${data.assetCode}</li>
                            <li><strong>Recipient:</strong> ${data.recipient}</li>
                            <li><strong>Transaction Hash:</strong> ${data.txHash}</li>
                        </ul>
                        <p>The transaction has been confirmed on the Stellar network.</p>
                        <p><a href="${data.explorerUrl}">View on Explorer</a></p>
                    `,
                    text: `Escrow ${data.escrowId} released: ${data.amount} ${data.assetCode} to ${data.recipient}.`,
                };

            case NotificationType.SECURITY_ALERT:
                return {
                    subject: "🔒 Security Alert - Action Required",
                    html: `
                        <h2 style="color: #d32f2f;">Security Alert</h2>
                        <p>Hi ${userName},</p>
                        <p><strong>We detected unusual activity on your account.</strong></p>
                        <ul>
                            <li><strong>Alert Type:</strong> ${data.alertType}</li>
                            <li><strong>Time:</strong> ${data.timestamp}</li>
                            <li><strong>IP Address:</strong> ${data.ipAddress}</li>
                            <li><strong>Location:</strong> ${data.location}</li>
                        </ul>
                        <p>If this was you, no action is needed. Otherwise, please secure your account immediately.</p>
                        <p><a href="${data.securityUrl}">Review Security Settings</a></p>
                    `,
                    text: `Security Alert: Unusual activity detected on your account at ${data.timestamp}. Review immediately.`,
                };

            case NotificationType.COLLATERAL_LOCKED:
                return {
                    subject: "🔐 Collateral Locked Successfully",
                    html: `
                        <h2>Collateral Locked</h2>
                        <p>Hi ${userName},</p>
                        <p>Your collateral has been successfully locked for the escrow.</p>
                        <ul>
                            <li><strong>Collateral ID:</strong> ${data.collateralId}</li>
                            <li><strong>Amount:</strong> ${data.amount} ${data.assetCode}</li>
                            <li><strong>Escrow ID:</strong> ${data.escrowId}</li>
                        </ul>
                        <p>Your collateral is now secured and will be released upon escrow completion.</p>
                    `,
                    text: `Collateral ${data.collateralId} locked for escrow ${data.escrowId}.`,
                };

            case NotificationType.ORACLE_CONFIRMATION:
                return {
                    subject: "✓ Oracle Confirmation Received",
                    html: `
                        <h2>Oracle Confirmation</h2>
                        <p>Hi ${userName},</p>
                        <p>An oracle has confirmed the event for your escrow.</p>
                        <ul>
                            <li><strong>Escrow ID:</strong> ${data.escrowId}</li>
                            <li><strong>Event Type:</strong> ${data.eventType}</li>
                            <li><strong>Oracle:</strong> ${data.oracleAddress}</li>
                            <li><strong>Status:</strong> ${data.status}</li>
                        </ul>
                        <p>The escrow will proceed according to the confirmation.</p>
                    `,
                    text: `Oracle confirmed ${data.eventType} for escrow ${data.escrowId}.`,
                };

            case NotificationType.GOVERNANCE_PROPOSAL:
                return {
                    subject: "🗳️ New Governance Proposal",
                    html: `
                        <h2>New Proposal to Vote On</h2>
                        <p>Hi ${userName},</p>
                        <p>A new governance proposal has been created.</p>
                        <ul>
                            <li><strong>Proposal ID:</strong> ${data.proposalId}</li>
                            <li><strong>Title:</strong> ${data.title}</li>
                            <li><strong>Voting Ends:</strong> ${data.endsAt}</li>
                        </ul>
                        <p>${data.description}</p>
                        <p><a href="${data.voteUrl}">Cast Your Vote</a></p>
                    `,
                    text: `New governance proposal: ${data.title}. Vote by ${data.endsAt}.`,
                };

            default:
                return {
                    subject: "Notification from StelloVault",
                    html: `<p>Hi ${userName},</p><p>You have a new notification.</p>`,
                    text: `Hi ${userName}, you have a new notification.`,
                };
        }
    }

    private async getUserPreferences(userId: string): Promise<Record<string, boolean>> {
        // In production, fetch from database
        // For now, return default preferences (all enabled)
        return {
            [NotificationType.LOAN_APPROVED]: true,
            [NotificationType.LOAN_REPAYMENT_DUE]: true,
            [NotificationType.ESCROW_EXPIRING_SOON]: true,
            [NotificationType.ESCROW_RELEASED]: true,
            [NotificationType.SECURITY_ALERT]: true,
            [NotificationType.COLLATERAL_LOCKED]: true,
            [NotificationType.ORACLE_CONFIRMATION]: true,
            [NotificationType.GOVERNANCE_PROPOSAL]: true,
        };
    }

    private shouldSendNotification(
        type: NotificationType,
        preferences: Record<string, boolean>
    ): boolean {
        // Security alerts always sent regardless of preferences
        if (type === NotificationType.SECURITY_ALERT) {
            return true;
        }

        return preferences[type] !== false;
    }

    private startProcessor(): void {
        // Process queue every 5 seconds
        setInterval(() => {
            this.processQueue();
        }, 5000);
    }

    /**
     * Update user notification preferences
     */
    async updatePreferences(
        userId: string,
        preferences: Partial<Record<NotificationType, boolean>>
    ): Promise<void> {
        // In production, save to database
        console.log(`Updated preferences for user ${userId}:`, preferences);
        this.emit("preferences:updated", { userId, preferences });
    }
}

export default new NotificationService();

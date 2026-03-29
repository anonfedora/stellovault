import webhookService from "./webhook.service";

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
  type: "PaymentReceived" | string;
  payload: Record<string, unknown>;
}

export class EventMonitoringService {
  private rpc: any; // Using Soroban RPC

  constructor() {
    // Initialize RPC client
  }

  /**
   * Polls the RPC for new events matching specific filters.
   */
  async pollEvents() {
    console.log("Polling for Soroban events...");
    // Logic to fetch events and update database
    // This bridges on-chain finality to off-chain DB
  }

  async processEvent(event: MonitoringEvent) {
    if (event.type === "PaymentReceived") {
      await this.handlePaymentReceived(
        event.payload as unknown as PaymentReceivedEventData,
      );
    }
  }

  private async handlePaymentReceived(event: PaymentReceivedEventData) {
    if (!event.webhookUrl) {
      return;
    }

    const webhookSecret = event.webhookSecret?.trim();
    if (!webhookSecret) {
      console.warn(
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
}

export default new EventMonitoringService();

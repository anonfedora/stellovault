// server/src/services/payment.service.ts
import { paymentEmitter, PAYMENT_UPDATED, PAYMENT_FINALIZED } from "../events/payment.emitter";

type PaymentStatus = "pending" | "processing" | "paid" | "failed";

interface Payment {
  id: string;
  status: PaymentStatus;
}

const payments = new Map<string, Payment>();

export class PaymentService {
  static getStatus(id: string): Payment | null {
    return payments.get(id) || null;
  }

  static updateStatus(id: string, status: PaymentStatus) {
    const payment = payments.get(id) || { id, status: "pending" };
    payment.status = status;

    payments.set(id, payment);

    paymentEmitter.emit(PAYMENT_UPDATED, payment);

    if (status === "paid" || status === "failed") {
      paymentEmitter.emit(PAYMENT_FINALIZED, payment);
    }
  }
}

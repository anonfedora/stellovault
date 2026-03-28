// server/src/events/payment.emitter.ts
import { EventEmitter } from "events";

class PaymentEmitter extends EventEmitter {}

export const paymentEmitter = new PaymentEmitter();

// event names
export const PAYMENT_UPDATED = "payment_updated";
export const PAYMENT_FINALIZED = "payment_finalized";

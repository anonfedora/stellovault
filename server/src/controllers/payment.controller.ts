// server/src/controllers/payment.controller.ts
import { Request, Response } from "express";
import { PaymentService } from "../services/payment.service";
import { paymentEmitter, PAYMENT_UPDATED, PAYMENT_FINALIZED } from "../events/payment.emitter";

export class PaymentController {
  static getStatus(req: Request, res: Response) {
    const { id } = req.params;

    const payment = PaymentService.getStatus(id);

    if (!payment) {
      return res.status(404).json({ message: "Payment not found" });
    }

    return res.json(payment);
  }

  static streamStatus(req: Request, res: Response) {
    const { id } = req.params;

    res.setHeader("Content-Type", "text/event-stream");
    res.setHeader("Cache-Control", "no-cache");
    res.setHeader("Connection", "keep-alive");

    const send = (data: any) => {
      res.write(`data: ${JSON.stringify(data)}\n\n`);
    };

    const onUpdate = (payment: any) => {
      if (payment.id === id) {
        send(payment);
      }
    };

    const onFinalize = (payment: any) => {
      if (payment.id === id) {
        send(payment);
        res.end(); // close connection when done
      }
    };

    paymentEmitter.on(PAYMENT_UPDATED, onUpdate);
    paymentEmitter.on(PAYMENT_FINALIZED, onFinalize);

    req.on("close", () => {
      paymentEmitter.off(PAYMENT_UPDATED, onUpdate);
      paymentEmitter.off(PAYMENT_FINALIZED, onFinalize);
    });
  }
}
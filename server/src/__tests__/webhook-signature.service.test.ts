import {
  signWebhookPayload,
  verifyWebhookSignature,
} from "../services/webhook-signature.service";

describe("webhook-signature.service", () => {
  it("signs and verifies a payload with HMAC-SHA256", () => {
    const payload = JSON.stringify({
      event: "payment.received",
      amount: "10.00",
    });
    const secret = "super-secret";
    const signature = signWebhookPayload(payload, secret);

    expect(signature.startsWith("sha256=")).toBe(true);
    expect(verifyWebhookSignature(payload, signature, secret)).toBe(true);
  });

  it("rejects a tampered payload", () => {
    const secret = "super-secret";
    const signature = signWebhookPayload(JSON.stringify({ ok: true }), secret);

    expect(
      verifyWebhookSignature(JSON.stringify({ ok: false }), signature, secret),
    ).toBe(false);
  });
});

"use client";

import { useState } from "react";

interface ConfirmationFormProps {
  oracleAddress: string;
  onSubmit: (data: {
    oracleAddress: string;
    escrowId: string;
    eventType: string;
    signature: string;
    payload: Record<string, unknown>;
    nonce: string;
  }) => Promise<void>;
  loading?: boolean;
}

const EVENT_TYPES = ["SHIPMENT_CONFIRMED", "QUALITY_VERIFIED", "DELIVERY_COMPLETED", "DISPUTE_RAISED"];

export function ConfirmationForm({ oracleAddress, onSubmit, loading = false }: ConfirmationFormProps) {
  const [escrowId, setEscrowId] = useState("");
  const [eventType, setEventType] = useState(EVENT_TYPES[0]);
  const [signature, setSignature] = useState("");
  const [payloadJson, setPayloadJson] = useState("{}");
  const [nonce, setNonce] = useState(() => crypto.randomUUID());
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    let payload: Record<string, unknown>;
    try {
      payload = JSON.parse(payloadJson);
    } catch {
      setError("Payload must be valid JSON");
      return;
    }
    try {
      await onSubmit({ oracleAddress, escrowId, eventType, signature, payload, nonce });
      setSuccess(true);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Submission failed");
    }
  }

  if (success) {
    return (
      <div className="rounded-lg border border-green-200 bg-green-50 p-4 text-sm text-green-800">
        Confirmation submitted successfully.
      </div>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <label className="block space-y-1 text-sm font-medium text-gray-700">
        Escrow ID
        <input
          required
          value={escrowId}
          onChange={(e) => setEscrowId(e.target.value)}
          placeholder="UUID of the escrow"
          className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
        />
      </label>
      <label className="block space-y-1 text-sm font-medium text-gray-700">
        Event Type
        <select
          value={eventType}
          onChange={(e) => setEventType(e.target.value)}
          className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
        >
          {EVENT_TYPES.map((t) => <option key={t}>{t}</option>)}
        </select>
      </label>
      <label className="block space-y-1 text-sm font-medium text-gray-700">
        Signature (hex or base64, 64 bytes)
        <input
          required
          value={signature}
          onChange={(e) => setSignature(e.target.value)}
          placeholder="Ed25519 signature"
          className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 font-mono text-xs"
        />
      </label>
      <label className="block space-y-1 text-sm font-medium text-gray-700">
        Payload (JSON)
        <textarea
          value={payloadJson}
          onChange={(e) => setPayloadJson(e.target.value)}
          rows={3}
          className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 font-mono text-xs"
        />
      </label>
      <div className="flex items-center justify-between text-xs text-gray-400">
        <span>Nonce: {nonce}</span>
        <button type="button" onClick={() => setNonce(crypto.randomUUID())} className="text-blue-600 hover:underline">
          Regenerate
        </button>
      </div>
      {error && <p className="text-xs text-red-600">{error}</p>}
      <button
        type="submit"
        disabled={loading}
        className="w-full rounded-lg bg-blue-900 py-2.5 text-sm font-semibold text-white disabled:opacity-50"
      >
        {loading ? "Submitting…" : "Submit Confirmation"}
      </button>
    </form>
  );
}

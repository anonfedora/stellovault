"use client";

import { useState } from "react";
import { CheckCircle2, ChevronRight } from "lucide-react";
import { DocumentUpload } from "./DocumentUpload";
import type { CollateralDocument, CollateralAssetType } from "@/hooks/useCollateral";

const ASSET_TYPES: CollateralAssetType[] = ["INVOICE", "COMMODITY", "RECEIVABLE", "INVENTORY", "REAL_ESTATE"];

const STEPS = ["Asset Details", "Documents", "Review & Tokenize"];

interface TokenizationFormProps {
  escrowId: string;
  onSubmit: (data: {
    escrowId: string;
    assetData: { assetType: string; amount: number; description: string; issuer: string };
    documents: CollateralDocument[];
  }) => Promise<void>;
  loading?: boolean;
}

export function TokenizationForm({ escrowId, onSubmit, loading = false }: TokenizationFormProps) {
  const [step, setStep] = useState(0);
  const [assetType, setAssetType] = useState<CollateralAssetType>("INVOICE");
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [issuer, setIssuer] = useState("");
  const [documents, setDocuments] = useState<CollateralDocument[]>([]);
  const [done, setDone] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit() {
    setError(null);
    if (documents.length === 0) {
      setError("Upload at least one supporting document before tokenizing.");
      setStep(1);
      return;
    }

    try {
      await onSubmit({
        escrowId,
        assetData: { assetType, amount: Number(amount), description, issuer },
        documents,
      });
      setDone(true);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Tokenization failed");
    }
  }

  if (done) {
    return (
      <div className="flex flex-col items-center gap-3 py-12 text-center">
        <CheckCircle2 className="h-12 w-12 text-green-500" />
        <h3 className="text-lg font-bold text-gray-900">Asset Tokenized</h3>
        <p className="text-sm text-gray-600">Your collateral token has been created on Stellar.</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Step indicator */}
      <div className="flex items-center gap-2">
        {STEPS.map((label, i) => (
          <div key={label} className="flex items-center gap-2">
            <div
              className={`flex h-7 w-7 items-center justify-center rounded-full text-xs font-bold ${i <= step ? "bg-blue-900 text-white" : "bg-gray-200 text-gray-500"}`}
            >
              {i + 1}
            </div>
            <span className={`hidden text-xs sm:block ${i === step ? "font-semibold text-gray-900" : "text-gray-400"}`}>
              {label}
            </span>
            {i < STEPS.length - 1 && <ChevronRight className="h-4 w-4 text-gray-300" />}
          </div>
        ))}
      </div>

      {/* Step 0 — Asset Details */}
      {step === 0 && (
        <div className="space-y-4">
          <div className="rounded-2xl border border-blue-100 bg-blue-50 p-4 text-sm text-blue-700">
            Supported collateral: invoices, commodities, receivables, inventory, and real estate.
          </div>
          <label className="block space-y-1 text-sm font-medium text-gray-700">
            Asset Type
            <select
              value={assetType}
              onChange={(e) => setAssetType(e.target.value as CollateralAssetType)}
              className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
            >
              {ASSET_TYPES.map((t) => <option key={t}>{t}</option>)}
            </select>
          </label>
          <label className="block space-y-1 text-sm font-medium text-gray-700">
            USD Value
            <input
              type="number"
              min="1"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="e.g. 50000"
              className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
            />
          </label>
          <label className="block space-y-1 text-sm font-medium text-gray-700">
            Description
            <input
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Brief description of the asset"
              className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
            />
          </label>
          <label className="block space-y-1 text-sm font-medium text-gray-700">
            Issuer / Owner
            <input
              type="text"
              value={issuer}
              onChange={(e) => setIssuer(e.target.value)}
              placeholder="Company or individual name"
              className="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 text-sm"
            />
          </label>
          <button
            type="button"
            disabled={!amount || Number(amount) <= 0}
            onClick={() => {
              if (!amount || Number(amount) <= 0) {
                setError('Enter a valid USD value to continue.');
                return;
              }

              setError(null);
              setStep(1);
            }}
            className="w-full rounded-lg bg-blue-900 py-2.5 text-sm font-semibold text-white disabled:opacity-50"
          >
            Next: Upload Documents
          </button>
        </div>
      )}

      {/* Step 1 — Documents */}
      {step === 1 && (
        <div className="space-y-4">
          <p className="text-sm text-gray-500">
            Upload proof documents such as invoices, certificates, and receipts. Accepted formats: PDF, JPEG, PNG, WEBP.
          </p>
          <DocumentUpload documents={documents} onChange={(docs) => { setDocuments(docs); if (docs.length) setError(null); }} />
          {documents.length === 0 && (
            <p className="text-xs text-red-600">Please upload at least one supporting document to continue.</p>
          )}
          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => setStep(0)}
              className="flex-1 rounded-lg border border-gray-300 py-2.5 text-sm font-semibold text-gray-700"
            >
              Back
            </button>
            <button
              type="button"
              onClick={() => {
                if (documents.length === 0) {
                  setError('Upload at least one supporting document before reviewing.');
                  return;
                }
                setError(null);
                setStep(2);
              }}
              className="flex-1 rounded-lg bg-blue-900 py-2.5 text-sm font-semibold text-white"
            >
              Next: Review
            </button>
          </div>
        </div>
      )}

      {/* Step 2 — Review */}
      {step === 2 && (
        <div className="space-y-4">
          <div className="rounded-lg border border-gray-200 bg-gray-50 p-4 text-sm space-y-2">
            <div className="flex justify-between"><span className="text-gray-500">Asset Type</span><span className="font-medium">{assetType}</span></div>
            <div className="flex justify-between"><span className="text-gray-500">USD Value</span><span className="font-medium">${Number(amount).toLocaleString()}</span></div>
            <div className="flex justify-between"><span className="text-gray-500">Description</span><span className="font-medium">{description || "—"}</span></div>
            <div className="flex justify-between"><span className="text-gray-500">Issuer</span><span className="font-medium">{issuer || "—"}</span></div>
            <div className="flex justify-between"><span className="text-gray-500">Documents</span><span className="font-medium">{documents.length}</span></div>
          </div>
          {error && <p className="text-xs text-red-600">{error}</p>}
          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => setStep(1)}
              className="flex-1 rounded-lg border border-gray-300 py-2.5 text-sm font-semibold text-gray-700"
            >
              Back
            </button>
            <button
              type="button"
              onClick={handleSubmit}
              disabled={loading}
              className="flex-1 rounded-lg bg-blue-900 py-2.5 text-sm font-semibold text-white disabled:opacity-50"
            >
              {loading ? "Tokenizing…" : "Tokenize Asset"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

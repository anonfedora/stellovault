"use client";

import { useMemo, useState } from "react";
import { ArrowLeft, ArrowRight, CheckCircle2 } from "lucide-react";

const steps = ["Parties", "Terms", "Review"];

export function EscrowForm() {
  const [step, setStep] = useState(0);
  const [form, setForm] = useState({
    buyer: "",
    seller: "",
    amount: "",
    asset: "USDC",
    dueDate: "",
    route: "",
    oracle: "Shipment Oracle A",
  });

  const canContinue = useMemo(() => {
    if (step === 0) return form.buyer.length > 2 && form.seller.length > 2;
    if (step === 1) return Number(form.amount) > 0 && form.dueDate && form.route.length > 2;
    return true;
  }, [form, step]);

  const updateField = (field: keyof typeof form, value: string) => {
    setForm((current) => ({ ...current, [field]: value }));
  };

  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4 shadow-sm sm:p-6">
      <div className="mb-6 grid grid-cols-3 gap-2">
        {steps.map((label, index) => (
          <div key={label} className="flex items-center gap-2">
            <span
              className={`flex h-8 w-8 items-center justify-center rounded-full text-sm font-bold ${
                index <= step ? "bg-blue-900 text-white" : "bg-gray-100 text-gray-500"
              }`}
            >
              {index < step ? <CheckCircle2 className="h-4 w-4" /> : index + 1}
            </span>
            <span className="hidden text-sm font-medium text-gray-700 sm:inline">{label}</span>
          </div>
        ))}
      </div>

      {step === 0 && (
        <div className="grid gap-4">
          <label className="grid gap-2 text-sm font-medium text-gray-700">
            Buyer
            <input
              value={form.buyer}
              onChange={(event) => updateField("buyer", event.target.value)}
              className="min-h-12 rounded-lg border border-gray-300 px-3 text-base text-gray-950 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
              placeholder="Buyer organization"
              autoComplete="organization"
            />
          </label>
          <label className="grid gap-2 text-sm font-medium text-gray-700">
            Seller
            <input
              value={form.seller}
              onChange={(event) => updateField("seller", event.target.value)}
              className="min-h-12 rounded-lg border border-gray-300 px-3 text-base text-gray-950 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
              placeholder="Seller organization"
              autoComplete="organization"
            />
          </label>
        </div>
      )}

      {step === 1 && (
        <div className="grid gap-4 sm:grid-cols-2">
          <label className="grid gap-2 text-sm font-medium text-gray-700">
            Amount
            <input
              value={form.amount}
              onChange={(event) => updateField("amount", event.target.value)}
              type="number"
              inputMode="decimal"
              className="min-h-12 rounded-lg border border-gray-300 px-3 text-base text-gray-950 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
              placeholder="0.00"
            />
          </label>
          <label className="grid gap-2 text-sm font-medium text-gray-700">
            Asset
            <select
              value={form.asset}
              onChange={(event) => updateField("asset", event.target.value)}
              className="min-h-12 rounded-lg border border-gray-300 px-3 text-base text-gray-950 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
            >
              <option>USDC</option>
              <option>XLM</option>
            </select>
          </label>
          <label className="grid gap-2 text-sm font-medium text-gray-700">
            Due date
            <input
              value={form.dueDate}
              onChange={(event) => updateField("dueDate", event.target.value)}
              type="date"
              className="min-h-12 rounded-lg border border-gray-300 px-3 text-base text-gray-950 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
            />
          </label>
          <label className="grid gap-2 text-sm font-medium text-gray-700">
            Oracle
            <select
              value={form.oracle}
              onChange={(event) => updateField("oracle", event.target.value)}
              className="min-h-12 rounded-lg border border-gray-300 px-3 text-base text-gray-950 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
            >
              <option>Shipment Oracle A</option>
              <option>IoT Seal Oracle</option>
              <option>Warehouse Oracle C</option>
            </select>
          </label>
          <label className="grid gap-2 text-sm font-medium text-gray-700 sm:col-span-2">
            Delivery route
            <input
              value={form.route}
              onChange={(event) => updateField("route", event.target.value)}
              className="min-h-12 rounded-lg border border-gray-300 px-3 text-base text-gray-950 outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-100"
              placeholder="Origin -> destination"
            />
          </label>
        </div>
      )}

      {step === 2 && (
        <div className="grid gap-3 rounded-lg bg-gray-50 p-4 text-sm text-gray-700">
          <div className="flex justify-between gap-4"><span>Buyer</span><strong>{form.buyer}</strong></div>
          <div className="flex justify-between gap-4"><span>Seller</span><strong>{form.seller}</strong></div>
          <div className="flex justify-between gap-4"><span>Amount</span><strong>{form.amount} {form.asset}</strong></div>
          <div className="flex justify-between gap-4"><span>Due date</span><strong>{form.dueDate}</strong></div>
          <div className="flex justify-between gap-4"><span>Oracle</span><strong>{form.oracle}</strong></div>
          <div className="flex justify-between gap-4"><span>Route</span><strong>{form.route}</strong></div>
        </div>
      )}

      <div className="mt-6 flex flex-col-reverse gap-3 sm:flex-row sm:justify-between">
        <button
          type="button"
          onClick={() => setStep((current) => Math.max(0, current - 1))}
          disabled={step === 0}
          className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg border border-gray-300 px-4 font-semibold text-gray-700 disabled:opacity-40"
        >
          <ArrowLeft className="h-4 w-4" />
          Back
        </button>
        <button
          type="button"
          onClick={() => (step === 2 ? undefined : setStep((current) => current + 1))}
          disabled={!canContinue}
          className="inline-flex min-h-12 items-center justify-center gap-2 rounded-lg bg-blue-900 px-4 font-semibold text-white disabled:opacity-40"
        >
          {step === 2 ? "Create escrow" : "Continue"}
          <ArrowRight className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}

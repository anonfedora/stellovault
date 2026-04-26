"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useGovernance } from "@/hooks/useGovernance";

const PROPOSAL_TYPES = [
  "Protocol Settings",
  "Reward Parameters",
  "Treasury Grant",
  "Community Initiative",
] as const;

const STEPS = ["Details", "Review"] as const;

interface FormData {
  title: string;
  description: string;
  type: string;
  duration: number;
}

const EMPTY: FormData = {
  title: "",
  description: "",
  type: "Protocol Settings",
  duration: 7,
};

function validate(data: FormData): Record<string, string> {
  const errors: Record<string, string> = {};
  if (!data.title.trim()) errors.title = "Title is required.";
  else if (data.title.trim().length < 10)
    errors.title = "Title must be at least 10 characters.";
  if (!data.description.trim()) errors.description = "Description is required.";
  else if (data.description.trim().length < 30)
    errors.description = "Description must be at least 30 characters.";
  if (data.duration < 1 || data.duration > 30)
    errors.duration = "Duration must be between 1 and 30 days.";
  return errors;
}

export function ProposalForm() {
  const router = useRouter();
  const { createProposal } = useGovernance();
  const [createdAt] = useState(() => Date.now());
  const [step, setStep] = useState<0 | 1>(0);
  const [formData, setFormData] = useState<FormData>(EMPTY);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const set = (field: keyof FormData, value: string | number) =>
    setFormData((prev) => ({ ...prev, [field]: value }));

  const handleNext = () => {
    const errs = validate(formData);
    if (Object.keys(errs).length > 0) {
      setErrors(errs);
      return;
    }
    setErrors({});
    setStep(1);
  };

  const handleSubmit = async () => {
    setSubmitting(true);
    setSubmitError(null);
    try {
      const proposal = await createProposal(formData);
      router.push(`/governance/${proposal.id}`);
    } catch (err: unknown) {
      setSubmitError(
        err instanceof Error ? err.message : "Failed to submit proposal.",
      );
      setSubmitting(false);
    }
  };

  const expiresAt = new Date(createdAt + 86400000 * formData.duration);

  return (
    <div className="min-h-screen bg-zinc-50 dark:bg-black py-12 px-4 sm:px-6">
      <div className="max-w-3xl mx-auto">
        {/* Back */}
        <div className="mb-8">
          <Link
            href="/governance"
            className="text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-50 transition-colors"
          >
            ← Cancel
          </Link>
        </div>

        {/* Step indicator */}
        <div className="flex items-center gap-3 mb-8">
          {STEPS.map((label, i) => (
            <div key={label} className="flex items-center gap-3">
              <div
                className={[
                  "flex h-8 w-8 items-center justify-center rounded-full text-sm font-bold transition-colors",
                  i === step
                    ? "bg-zinc-900 text-white dark:bg-white dark:text-zinc-900"
                    : i < step
                      ? "bg-emerald-500 text-white"
                      : "bg-zinc-200 text-zinc-500 dark:bg-zinc-800 dark:text-zinc-400",
                ].join(" ")}
              >
                {i < step ? (
                  <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                  </svg>
                ) : (
                  i + 1
                )}
              </div>
              <span
                className={[
                  "text-sm font-semibold",
                  i === step
                    ? "text-zinc-900 dark:text-zinc-50"
                    : "text-zinc-400 dark:text-zinc-500",
                ].join(" ")}
              >
                {label}
              </span>
              {i < STEPS.length - 1 && (
                <div className="h-px w-8 bg-zinc-200 dark:bg-zinc-800" />
              )}
            </div>
          ))}
        </div>

        <div className="bg-white dark:bg-zinc-900 rounded-3xl border border-zinc-200 dark:border-zinc-800 p-8 shadow-sm">
          {step === 0 && (
            <>
              <h1 className="text-3xl font-extrabold mb-8 text-zinc-900 dark:text-zinc-50">
                Proposal Details
              </h1>

              <div className="space-y-6">
                {/* Title */}
                <div className="space-y-2">
                  <label
                    htmlFor="title"
                    className="text-xs font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
                  >
                    Title
                  </label>
                  <input
                    id="title"
                    value={formData.title}
                    onChange={(e) => set("title", e.target.value)}
                    className={[
                      "w-full px-6 py-4 rounded-2xl border bg-zinc-50 dark:bg-zinc-950/50 text-xl font-semibold focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all",
                      errors.title
                        ? "border-rose-400 dark:border-rose-600"
                        : "border-zinc-200 dark:border-zinc-800",
                    ].join(" ")}
                    placeholder="Brief title for your proposal"
                  />
                  {errors.title && (
                    <p className="text-xs text-rose-500 font-medium">{errors.title}</p>
                  )}
                </div>

                {/* Type + Duration */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="space-y-2">
                    <label
                      htmlFor="type"
                      className="text-xs font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
                    >
                      Type
                    </label>
                    <select
                      id="type"
                      value={formData.type}
                      onChange={(e) => set("type", e.target.value)}
                      className="w-full px-6 py-4 rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-950/50 font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 appearance-none"
                    >
                      {PROPOSAL_TYPES.map((t) => (
                        <option key={t}>{t}</option>
                      ))}
                    </select>
                  </div>

                  <div className="space-y-2">
                    <label
                      htmlFor="duration"
                      className="text-xs font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
                    >
                      Duration (Days)
                    </label>
                    <input
                      id="duration"
                      type="number"
                      min="1"
                      max="30"
                      value={formData.duration}
                      onChange={(e) =>
                        set("duration", parseInt(e.target.value) || 1)
                      }
                      className={[
                        "w-full px-6 py-4 rounded-2xl border bg-zinc-50 dark:bg-zinc-950/50 font-medium focus:outline-none focus:ring-2 focus:ring-blue-500",
                        errors.duration
                          ? "border-rose-400 dark:border-rose-600"
                          : "border-zinc-200 dark:border-zinc-800",
                      ].join(" ")}
                    />
                    {errors.duration && (
                      <p className="text-xs text-rose-500 font-medium">{errors.duration}</p>
                    )}
                  </div>
                </div>

                {/* Description */}
                <div className="space-y-2">
                  <label
                    htmlFor="description"
                    className="text-xs font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
                  >
                    Detailed Description
                  </label>
                  <textarea
                    id="description"
                    rows={8}
                    value={formData.description}
                    onChange={(e) => set("description", e.target.value)}
                    className={[
                      "w-full px-6 py-4 rounded-2xl border bg-zinc-50 dark:bg-zinc-950/50 font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all resize-none",
                      errors.description
                        ? "border-rose-400 dark:border-rose-600"
                        : "border-zinc-200 dark:border-zinc-800",
                    ].join(" ")}
                    placeholder="Explain the motivation and details of your proposal..."
                  />
                  <div className="flex justify-between">
                    {errors.description ? (
                      <p className="text-xs text-rose-500 font-medium">{errors.description}</p>
                    ) : (
                      <span />
                    )}
                    <span className="text-xs text-zinc-400">
                      {formData.description.length} chars
                    </span>
                  </div>
                </div>

                <button
                  type="button"
                  onClick={handleNext}
                  className="w-full h-14 rounded-2xl bg-zinc-900 dark:bg-white text-white dark:text-zinc-950 font-bold text-base hover:opacity-90 transition-all active:scale-[0.98]"
                >
                  Review Proposal →
                </button>
              </div>
            </>
          )}

          {step === 1 && (
            <>
              <h1 className="text-3xl font-extrabold mb-8 text-zinc-900 dark:text-zinc-50">
                Review & Submit
              </h1>

              <div className="space-y-6">
                {/* Summary card */}
                <div className="rounded-2xl border border-zinc-200 dark:border-zinc-800 divide-y divide-zinc-100 dark:divide-zinc-800 overflow-hidden">
                  <div className="px-6 py-4 flex justify-between items-start">
                    <span className="text-xs font-bold text-zinc-400 uppercase tracking-widest">Title</span>
                    <span className="text-sm font-semibold text-zinc-900 dark:text-zinc-100 text-right max-w-xs">
                      {formData.title}
                    </span>
                  </div>
                  <div className="px-6 py-4 flex justify-between">
                    <span className="text-xs font-bold text-zinc-400 uppercase tracking-widest">Type</span>
                    <span className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                      {formData.type}
                    </span>
                  </div>
                  <div className="px-6 py-4 flex justify-between">
                    <span className="text-xs font-bold text-zinc-400 uppercase tracking-widest">Duration</span>
                    <span className="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
                      {formData.duration} days
                    </span>
                  </div>
                  <div className="px-6 py-4">
                    <span className="text-xs font-bold text-zinc-400 uppercase tracking-widest block mb-2">
                      Description
                    </span>
                    <p className="text-sm text-zinc-700 dark:text-zinc-300 leading-relaxed whitespace-pre-wrap">
                      {formData.description}
                    </p>
                  </div>
                </div>

                {/* Wallet signing notice */}
                <div className="flex items-start gap-3 p-4 rounded-xl bg-blue-50 dark:bg-blue-950/20 border border-blue-100 dark:border-blue-900/30">
                  <svg className="h-5 w-5 text-blue-500 shrink-0 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  <p className="text-sm text-blue-700 dark:text-blue-300">
                    Submitting will prompt your Freighter wallet to sign the proposal XDR transaction on-chain.
                  </p>
                </div>

                {submitError && (
                  <div className="p-4 rounded-xl bg-rose-50 dark:bg-rose-950/20 border border-rose-100 dark:border-rose-900/30 text-rose-600 dark:text-rose-400 text-sm font-medium">
                    {submitError}
                  </div>
                )}

                <div className="flex gap-3">
                  <button
                    type="button"
                    onClick={() => setStep(0)}
                    disabled={submitting}
                    className="flex-1 h-14 rounded-2xl border border-zinc-200 dark:border-zinc-800 font-bold text-base hover:bg-zinc-50 dark:hover:bg-zinc-800 transition-all disabled:opacity-50"
                  >
                    ← Edit
                  </button>
                  <button
                    type="button"
                    onClick={handleSubmit}
                    disabled={submitting}
                    className="flex-[2] h-14 rounded-2xl bg-zinc-900 dark:bg-white text-white dark:text-zinc-950 font-bold text-base hover:opacity-90 transition-all active:scale-[0.98] disabled:opacity-50"
                  >
                    {submitting ? (
                      <span className="flex items-center justify-center gap-2">
                        <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                        </svg>
                        Signing & Submitting...
                      </span>
                    ) : (
                      "Submit Proposal"
                    )}
                  </button>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

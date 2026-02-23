"use client";

import { useRouter } from "next/navigation";
import { useGovernance } from "@/hooks/useGovernance";
import { useState } from "react";
import Link from "next/link";

export default function NewProposalPage() {
  const router = useRouter();
  const { createProposal, walletConnected } = useGovernance();
  const [loading, setLoading] = useState(false);
  const [formData, setFormData] = useState({
    title: "",
    description: "",
    type: "Protocol Settings",
    duration: 7,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!walletConnected) {
      alert("Keep your wallet connected to submit a proposal.");
      return;
    }

    setLoading(true);
    try {
      const p = await createProposal(formData);
      router.push(`/governance/${p.id}`);
    } catch (err) {
      console.error(err);
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-zinc-50 dark:bg-black py-12 px-4 sm:px-6">
      <div className="max-w-3xl mx-auto">
        <div className="mb-8">
          <Link
            href="/governance"
            className="text-sm font-semibold text-zinc-500 hover:text-zinc-900 dark:hover:text-zinc-50 transition-colors"
          >
            ← Cancel
          </Link>
        </div>

        <div className="bg-white dark:bg-zinc-900 rounded-3xl border border-zinc-200 dark:border-zinc-800 p-8 shadow-sm">
          <h1 className="text-3xl font-extrabold mb-8 text-zinc-900 dark:text-zinc-50">
            Create New Proposal
          </h1>

          <form onSubmit={handleSubmit} className="space-y-8">
            <div className="space-y-2">
              <label
                htmlFor="title"
                className="text-sm font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
              >
                Title
              </label>
              <input
                id="title"
                required
                value={formData.title}
                onChange={(e) =>
                  setFormData({ ...formData, title: e.target.value })
                }
                className="w-full px-6 py-4 rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-950/50 text-xl font-semibold focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                placeholder="Brief title for your proposal"
              />
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="space-y-2">
                <label
                  htmlFor="type"
                  className="text-sm font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
                >
                  Type
                </label>
                <select
                  id="type"
                  value={formData.type}
                  onChange={(e) =>
                    setFormData({ ...formData, type: e.target.value })
                  }
                  className="w-full px-6 py-4 rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-950/50 font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 appearance-none"
                >
                  <option>Protocol Settings</option>
                  <option>Reward Parameters</option>
                  <option>Treasury Grant</option>
                  <option>Community Initiative</option>
                </select>
              </div>

              <div className="space-y-2">
                <label
                  htmlFor="duration"
                  className="text-sm font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
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
                    setFormData({
                      ...formData,
                      duration: parseInt(e.target.value),
                    })
                  }
                  className="w-full px-6 py-4 rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-950/50 font-medium focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>

            <div className="space-y-2">
              <label
                htmlFor="description"
                className="text-sm font-bold text-zinc-500 dark:text-zinc-400 uppercase tracking-widest"
              >
                Detailed Description
              </label>
              <textarea
                id="description"
                required
                rows={8}
                value={formData.description}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                className="w-full px-6 py-4 rounded-2xl border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-950/50 font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all resize-none"
                placeholder="Explain the motivation and details of your proposal..."
              />
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full h-16 rounded-2xl bg-zinc-900 dark:bg-white text-white dark:text-zinc-950 font-bold text-lg hover:opacity-90 transition-all active:scale-[0.98] disabled:opacity-50"
            >
              {loading ? "Submitting..." : "Submit Proposal"}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
}

"use client";

import type { VoteType } from "@/hooks/useGovernance";

interface VoteButtonProps {
  type: VoteType;
  onClick: () => void;
  isLoading: boolean;
  disabled: boolean;
  /** The vote the user already cast, if any */
  userVote?: VoteType;
}

const STYLES: Record<VoteType, string> = {
  For: "bg-emerald-500 hover:bg-emerald-600 text-white dark:bg-emerald-600 dark:hover:bg-emerald-700 border-emerald-500",
  Against:
    "bg-rose-500 hover:bg-rose-600 text-white dark:bg-rose-600 dark:hover:bg-rose-700 border-rose-500",
  Abstain:
    "bg-zinc-200 hover:bg-zinc-300 text-zinc-900 dark:bg-zinc-800 dark:hover:bg-zinc-700 dark:text-zinc-100 border-zinc-300 dark:border-zinc-700",
};

const SELECTED_STYLES: Record<VoteType, string> = {
  For: "ring-2 ring-emerald-500 ring-offset-2 dark:ring-offset-zinc-900",
  Against: "ring-2 ring-rose-500 ring-offset-2 dark:ring-offset-zinc-900",
  Abstain: "ring-2 ring-zinc-400 ring-offset-2 dark:ring-offset-zinc-900",
};

export function VoteButton({
  type,
  onClick,
  isLoading,
  disabled,
  userVote,
}: VoteButtonProps) {
  const isSelected = userVote === type;

  return (
    <button
      onClick={onClick}
      disabled={disabled || isLoading}
      aria-pressed={isSelected}
      className={[
        "flex items-center justify-center gap-2 w-full px-6 py-2.5 rounded-xl font-semibold transition-all",
        "disabled:opacity-50 disabled:cursor-not-allowed active:scale-95",
        STYLES[type],
        isSelected ? SELECTED_STYLES[type] : "",
      ].join(" ")}
    >
      {isLoading ? (
        <span className="flex items-center gap-2">
          <svg className="animate-spin h-4 w-4 text-current" viewBox="0 0 24 24">
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
              fill="none"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          Signing...
        </span>
      ) : (
        <span className="flex items-center gap-1.5">
          {isSelected && (
            <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
            </svg>
          )}
          Vote {type}
        </span>
      )}
    </button>
  );
}

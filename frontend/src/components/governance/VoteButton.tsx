"use client";

import { VoteType } from "@/hooks/useGovernance";

interface VoteButtonProps {
  type: VoteType;
  onClick: () => void;
  isLoading: boolean;
  disabled: boolean;
}

export function VoteButton({
  type,
  onClick,
  isLoading,
  disabled,
}: VoteButtonProps) {
  const getStyles = () => {
    switch (type) {
      case "For":
        return "bg-emerald-500 hover:bg-emerald-600 text-white dark:bg-emerald-600 dark:hover:bg-emerald-700";
      case "Against":
        return "bg-rose-500 hover:bg-rose-600 text-white dark:bg-rose-600 dark:hover:bg-rose-700";
      case "Abstain":
        return "bg-zinc-200 hover:bg-zinc-300 text-zinc-900 dark:bg-zinc-800 dark:hover:bg-zinc-700 dark:text-zinc-100";
    }
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled || isLoading}
      className={`
        flex items-center justify-center gap-2 w-full px-6 py-2.5 rounded-xl font-semibold transition-all
        disabled:opacity-50 disabled:cursor-not-allowed active:scale-95
        ${getStyles()}
      `}
    >
      {isLoading ? (
        <span className="flex items-center gap-2">
          <svg
            className="animate-spin h-4 w-4 text-current"
            viewBox="0 0 24 24"
          >
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
        <span>Vote {type}</span>
      )}
    </button>
  );
}

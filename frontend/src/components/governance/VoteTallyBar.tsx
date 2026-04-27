"use client";

interface VoteTallyBarProps {
  votes: {
    for: number;
    against: number;
    abstain: number;
  };
  votingPower?: number;
  threshold?: number;
  showCounts?: boolean;
}

function formatVotes(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toLocaleString();
}

export function VoteTallyBar({
  votes,
  votingPower,
  threshold = 0.667,
  showCounts = true,
}: VoteTallyBarProps) {
  const total = votes.for + votes.against + votes.abstain;

  const forPercent = total > 0 ? (votes.for / total) * 100 : 0;
  const againstPercent = total > 0 ? (votes.against / total) * 100 : 0;
  const abstainPercent = total > 0 ? (votes.abstain / total) * 100 : 0;

  // Participation rate relative to total voting power
  const participation =
    votingPower && votingPower > 0 ? (total / votingPower) * 100 : null;

  // Threshold marker position (% of the for-segment)
  const thresholdPos = threshold * 100;

  return (
    <div className="space-y-4">
      {/* Tally bar */}
      <div className="relative">
        <div className="flex h-4 w-full overflow-hidden rounded-full bg-zinc-100 dark:bg-zinc-800">
          {total === 0 ? (
            <div className="h-full w-full bg-zinc-100 dark:bg-zinc-800" />
          ) : (
            <>
              <div
                className="h-full bg-emerald-500 transition-all duration-700"
                style={{ width: `${forPercent}%` }}
                title={`For: ${votes.for.toLocaleString()}`}
              />
              <div
                className="h-full bg-rose-500 transition-all duration-700"
                style={{ width: `${againstPercent}%` }}
                title={`Against: ${votes.against.toLocaleString()}`}
              />
              <div
                className="h-full bg-zinc-400 dark:bg-zinc-500 transition-all duration-700"
                style={{ width: `${abstainPercent}%` }}
                title={`Abstain: ${votes.abstain.toLocaleString()}`}
              />
            </>
          )}
        </div>

        {/* Threshold marker */}
        {total > 0 && (
          <div
            className="absolute top-0 h-4 w-0.5 bg-zinc-900 dark:bg-white opacity-60"
            style={{ left: `${thresholdPos}%` }}
            title={`Pass threshold: ${(threshold * 100).toFixed(1)}%`}
          />
        )}
      </div>

      {/* Legend */}
      <div className="grid grid-cols-3 gap-2 text-xs">
        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-1.5">
            <div className="h-2 w-2 shrink-0 rounded-full bg-emerald-500" />
            <span className="font-semibold text-zinc-700 dark:text-zinc-300">For</span>
          </div>
          <span className="text-zinc-500 dark:text-zinc-400 font-medium">
            {forPercent.toFixed(1)}%
          </span>
          {showCounts && (
            <span className="text-zinc-400 dark:text-zinc-500 text-[11px]">
              {formatVotes(votes.for)}
            </span>
          )}
        </div>

        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-1.5">
            <div className="h-2 w-2 shrink-0 rounded-full bg-rose-500" />
            <span className="font-semibold text-zinc-700 dark:text-zinc-300">Against</span>
          </div>
          <span className="text-zinc-500 dark:text-zinc-400 font-medium">
            {againstPercent.toFixed(1)}%
          </span>
          {showCounts && (
            <span className="text-zinc-400 dark:text-zinc-500 text-[11px]">
              {formatVotes(votes.against)}
            </span>
          )}
        </div>

        <div className="flex flex-col gap-1">
          <div className="flex items-center gap-1.5">
            <div className="h-2 w-2 shrink-0 rounded-full bg-zinc-400 dark:bg-zinc-500" />
            <span className="font-semibold text-zinc-700 dark:text-zinc-300">Abstain</span>
          </div>
          <span className="text-zinc-500 dark:text-zinc-400 font-medium">
            {abstainPercent.toFixed(1)}%
          </span>
          {showCounts && (
            <span className="text-zinc-400 dark:text-zinc-500 text-[11px]">
              {formatVotes(votes.abstain)}
            </span>
          )}
        </div>
      </div>

      {/* Participation */}
      {participation !== null && (
        <div className="pt-2 border-t border-zinc-100 dark:border-zinc-800 flex justify-between text-xs text-zinc-500 dark:text-zinc-400">
          <span>Participation</span>
          <span className="font-semibold text-zinc-700 dark:text-zinc-300">
            {participation.toFixed(1)}% of {formatVotes(votingPower!)} total
          </span>
        </div>
      )}
    </div>
  );
}

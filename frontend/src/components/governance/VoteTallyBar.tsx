"use client";

interface VoteTallyBarProps {
  votes: {
    for: number;
    against: number;
    abstain: number;
  };
}

export function VoteTallyBar({ votes }: VoteTallyBarProps) {
  const total = votes.for + votes.against + votes.abstain;

  if (total === 0) {
    return (
      <div className="h-3 w-full overflow-hidden rounded-full bg-zinc-100 dark:bg-zinc-800" />
    );
  }

  const forPercent = (votes.for / total) * 100;
  const againstPercent = (votes.against / total) * 100;
  const abstainPercent = (votes.abstain / total) * 100;

  return (
    <div className="space-y-2">
      <div className="flex h-3 w-full overflow-hidden rounded-full bg-zinc-100 dark:bg-zinc-800">
        <div
          className="h-full bg-emerald-500 transition-all duration-500"
          style={{ width: `${forPercent}%` }}
          title={`For: ${votes.for.toLocaleString()}`}
        />
        <div
          className="h-full bg-rose-500 transition-all duration-500"
          style={{ width: `${againstPercent}%` }}
          title={`Against: ${votes.against.toLocaleString()}`}
        />
        <div
          className="h-full bg-zinc-400 dark:bg-zinc-500 transition-all duration-500"
          style={{ width: `${abstainPercent}%` }}
          title={`Abstain: ${votes.abstain.toLocaleString()}`}
        />
      </div>
      <div className="flex justify-between text-xs font-medium text-zinc-500 dark:text-zinc-400">
        <div className="flex items-center gap-1.5">
          <div className="h-2 w-2 rounded-full bg-emerald-500" />
          <span>For: {forPercent.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="h-2 w-2 rounded-full bg-rose-500" />
          <span>Against: {againstPercent.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="h-2 w-2 rounded-full bg-zinc-400 dark:bg-zinc-500" />
          <span>Abstain: {abstainPercent.toFixed(1)}%</span>
        </div>
      </div>
    </div>
  );
}

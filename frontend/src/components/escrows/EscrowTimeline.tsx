import { CheckCircle2, Circle, Clock } from "lucide-react";
import type { EscrowEvent } from "@/hooks/useEscrows";
import { StatusBadge } from "./StatusBadge";

export function EscrowTimeline({ events }: { events: EscrowEvent[] }) {
  return (
    <ol className="snap-x space-y-4 overflow-x-auto pb-2 sm:overflow-visible">
      {events.map((event, index) => {
        const isLast = index === events.length - 1;

        return (
          <li key={event.id} className="relative flex min-w-[280px] snap-start gap-3 sm:min-w-0">
            {!isLast && (
              <span className="absolute left-4 top-9 h-[calc(100%-1rem)] w-px bg-gray-200" />
            )}
            <span className="relative z-10 flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-white ring-1 ring-gray-200">
              {isLast ? (
                <Clock className="h-4 w-4 text-amber-600" />
              ) : (
                <CheckCircle2 className="h-4 w-4 text-emerald-600" />
              )}
            </span>
            <div className="w-full rounded-lg border border-gray-200 bg-white p-4">
              <div className="flex flex-wrap items-start justify-between gap-2">
                <div>
                  <h3 className="font-semibold text-gray-950">{event.title}</h3>
                  <p className="mt-1 text-sm text-gray-600">{event.description}</p>
                </div>
                <StatusBadge status={event.status} />
              </div>
              <time className="mt-3 block text-xs font-medium text-gray-500">
                {new Date(event.timestamp).toLocaleString()}
              </time>
            </div>
          </li>
        );
      })}
      {events.length === 0 && (
        <li className="flex items-center gap-2 rounded-lg border border-dashed border-gray-300 p-4 text-sm text-gray-500">
          <Circle className="h-4 w-4" />
          No escrow events yet.
        </li>
      )}
    </ol>
  );
}

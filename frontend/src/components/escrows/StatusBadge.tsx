import type { EscrowStatus } from "@/hooks/useEscrows";

const statusStyles: Record<EscrowStatus, string> = {
  draft: "bg-gray-100 text-gray-700 ring-gray-200",
  funded: "bg-blue-100 text-blue-800 ring-blue-200",
  in_transit: "bg-cyan-100 text-cyan-800 ring-cyan-200",
  inspection: "bg-amber-100 text-amber-800 ring-amber-200",
  released: "bg-emerald-100 text-emerald-800 ring-emerald-200",
  disputed: "bg-red-100 text-red-800 ring-red-200",
};

const statusLabels: Record<EscrowStatus, string> = {
  draft: "Draft",
  funded: "Funded",
  in_transit: "In transit",
  inspection: "Inspection",
  released: "Released",
  disputed: "Disputed",
};

export function StatusBadge({ status }: { status: EscrowStatus }) {
  return (
    <span
      className={`inline-flex min-h-8 items-center rounded-full px-3 text-xs font-semibold ring-1 ${statusStyles[status]}`}
    >
      {statusLabels[status]}
    </span>
  );
}

export { statusLabels };

'use client';

import React, { useEffect, useState, useCallback, useRef } from 'react';

export interface TimelineLoan {
  id: string;
  principalAmount: number;
  interestRate: number; // basis points
  status: 'active' | 'repaid' | 'defaulted' | 'pending';
  createdAt: Date;
  dueAt: Date;
  endAt: Date; // dueAt for active/pending, updatedAt for completed
}

export interface LoanTimelineProps {
  walletAddress: string;
}

interface ApiLoan {
  id: string;
  principal_amount: number;
  interest_rate: number;
  status: string;
  created_at: string;
  due_at: string;
  updated_at: string;
}

interface TooltipState {
  loan: TimelineLoan;
  x: number;
  y: number;
}

const STATUS_COLORS: Record<TimelineLoan['status'], string> = {
  active: '#3b82f6',
  repaid: '#22c55e',
  defaulted: '#ef4444',
  pending: '#f59e0b',
};

const BAR_HEIGHT = 18;
const BAR_SPACING = 28;
const AXIS_PADDING = 40;
const MIN_BAR_WIDTH = 4;
const COMPRESS_THRESHOLD = 20;

function parseStatus(raw: string): TimelineLoan['status'] {
  const lower = raw.toLowerCase();
  if (lower === 'active' || lower === 'repaid' || lower === 'defaulted' || lower === 'pending') {
    return lower as TimelineLoan['status'];
  }
  return 'pending';
}

function mapApiLoan(raw: ApiLoan): TimelineLoan {
  const status = parseStatus(raw.status);
  const dueAt = new Date(raw.due_at);
  const updatedAt = new Date(raw.updated_at);
  const endAt = status === 'active' || status === 'pending' ? dueAt : updatedAt;
  return {
    id: raw.id,
    principalAmount: raw.principal_amount,
    interestRate: raw.interest_rate,
    status,
    createdAt: new Date(raw.created_at),
    dueAt,
    endAt,
  };
}

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(amount);
}

function formatDate(date: Date): string {
  return date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

function formatRate(basisPoints: number): string {
  return (basisPoints / 100).toFixed(2) + '%';
}

const LoanTimeline: React.FC<LoanTimelineProps> = ({ walletAddress }) => {
  const [loans, setLoans] = useState<TimelineLoan[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tooltip, setTooltip] = useState<TooltipState | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  const fetchLoans = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`/api/loans?wallet=${encodeURIComponent(walletAddress)}`);
      if (!res.ok) throw new Error(`Failed to fetch loans: ${res.status}`);
      const data: ApiLoan[] = await res.json();
      setLoans(data.map(mapApiLoan));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load loan history');
    } finally {
      setLoading(false);
    }
  }, [walletAddress]);

  useEffect(() => {
    fetchLoans();
  }, [fetchLoans]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-24 text-gray-500 text-sm">
        Loading loan history…
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center gap-2 py-6 text-sm">
        <span className="text-red-500">{error}</span>
        <button
          onClick={fetchLoans}
          className="px-3 py-1 rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors"
        >
          Retry
        </button>
      </div>
    );
  }

  if (loans.length === 0) {
    return (
      <div className="flex items-center justify-center h-24 text-gray-500 text-sm">
        No loan history found.
      </div>
    );
  }

  const numLoans = loans.length;
  const svgHeight = Math.max(120, numLoans * BAR_SPACING + AXIS_PADDING);
  const viewBoxWidth = 800;

  const timeMin = Math.min(...loans.map((l) => l.createdAt.getTime()));
  const timeMax = Math.max(...loans.map((l) => l.endAt.getTime()));
  const totalSpan = timeMax - timeMin || 1;

  // When >= COMPRESS_THRESHOLD loans, the axis is already proportionally compressed
  // because we use a fixed viewBox width — bars just get narrower naturally.
  // The compress flag is kept for potential future use but the SVG viewBox handles it.
  const _compress = numLoans >= COMPRESS_THRESHOLD; // eslint-disable-line @typescript-eslint/no-unused-vars

  const leftPad = 8;
  const rightPad = 8;
  const axisWidth = viewBoxWidth - leftPad - rightPad;

  function timeToX(t: number): number {
    return leftPad + ((t - timeMin) / totalSpan) * axisWidth;
  }

  function loanBarWidth(loan: TimelineLoan): number {
    const raw = ((loan.endAt.getTime() - loan.createdAt.getTime()) / totalSpan) * axisWidth;
    return Math.max(raw, MIN_BAR_WIDTH);
  }

  const axisY = svgHeight - 20;
  const startDate = new Date(timeMin);
  const endDate = new Date(timeMax);

  return (
    <div className="relative w-full">
      <svg
        ref={svgRef}
        viewBox={`0 0 ${viewBoxWidth} ${svgHeight}`}
        width="100%"
        height={svgHeight}
        aria-label="Loan timeline"
        role="img"
      >
        {/* Loan bars */}
        {loans.map((loan, i) => {
          const x = timeToX(loan.createdAt.getTime());
          const w = loanBarWidth(loan);
          const y = i * BAR_SPACING + 10;
          const color = STATUS_COLORS[loan.status];

          return (
            <rect
              key={loan.id}
              x={x}
              y={y}
              width={w}
              height={BAR_HEIGHT}
              fill={color}
              rx={3}
              ry={3}
              tabIndex={0}
              aria-label={`Loan ${loan.id.slice(0, 8)}, ${loan.status}`}
              style={{ cursor: 'pointer', outline: 'none' }}
              onMouseEnter={(e) => {
                const rect = (e.target as SVGRectElement).getBoundingClientRect();
                setTooltip({
                  loan,
                  x: rect.left + rect.width / 2,
                  y: rect.top,
                });
              }}
              onMouseLeave={() => setTooltip(null)}
              onFocus={(e) => {
                const rect = (e.target as SVGRectElement).getBoundingClientRect();
                setTooltip({
                  loan,
                  x: rect.left + rect.width / 2,
                  y: rect.top,
                });
              }}
              onBlur={() => setTooltip(null)}
            />
          );
        })}

        {/* Time axis line */}
        <line
          x1={leftPad}
          y1={axisY}
          x2={viewBoxWidth - rightPad}
          y2={axisY}
          stroke="#d1d5db"
          strokeWidth={1}
        />

        {/* Start tick */}
        <line x1={leftPad} y1={axisY - 4} x2={leftPad} y2={axisY + 4} stroke="#9ca3af" strokeWidth={1} />
        <text x={leftPad} y={axisY + 14} fontSize={9} fill="#6b7280" textAnchor="start">
          {formatDate(startDate)}
        </text>

        {/* End tick */}
        <line
          x1={viewBoxWidth - rightPad}
          y1={axisY - 4}
          x2={viewBoxWidth - rightPad}
          y2={axisY + 4}
          stroke="#9ca3af"
          strokeWidth={1}
        />
        <text x={viewBoxWidth - rightPad} y={axisY + 14} fontSize={9} fill="#6b7280" textAnchor="end">
          {formatDate(endDate)}
        </text>
      </svg>

      {/* Tooltip overlay */}
      {tooltip && (
        <div
          role="tooltip"
          style={{
            position: 'fixed',
            left: tooltip.x,
            top: tooltip.y - 8,
            transform: 'translate(-50%, -100%)',
            zIndex: 50,
            pointerEvents: 'none',
          }}
          className="bg-white border border-gray-200 rounded-lg shadow-lg px-3 py-2 text-xs text-gray-700 min-w-[180px]"
        >
          <div className="font-semibold text-gray-900 mb-1">
            {tooltip.loan.id.slice(0, 8)}
          </div>
          <div className="space-y-0.5">
            <div>
              <span className="text-gray-500">Principal: </span>
              {formatCurrency(tooltip.loan.principalAmount)}
            </div>
            <div>
              <span className="text-gray-500">Rate: </span>
              {formatRate(tooltip.loan.interestRate)}
            </div>
            <div>
              <span className="text-gray-500">Status: </span>
              <span
                style={{ color: STATUS_COLORS[tooltip.loan.status] }}
                className="font-medium capitalize"
              >
                {tooltip.loan.status}
              </span>
            </div>
            <div>
              <span className="text-gray-500">Start: </span>
              {formatDate(tooltip.loan.createdAt)}
            </div>
            <div>
              <span className="text-gray-500">Due: </span>
              {formatDate(tooltip.loan.dueAt)}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default LoanTimeline;

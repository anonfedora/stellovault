export type QuickStartStep =
  | "connectWallet"
  | "uploadCollateral"
  | "createEscrow"
  | "monitorLoan";

export const ONBOARDING_STORAGE_KEYS = {
  dismissed: "sv_onboarding_dismissed",
  completed: "sv_onboarding_completed",
  transactionCount: "sv_transaction_count",
  quickStart: "sv_quick_start",
} as const;

type QuickStartState = Record<QuickStartStep, boolean>;

const DEFAULT_QUICK_START: QuickStartState = {
  connectWallet: false,
  uploadCollateral: false,
  createEscrow: false,
  monitorLoan: false,
};

function safeJsonParse<T>(value: string | null): T | null {
  if (!value) return null;
  try {
    return JSON.parse(value) as T;
  } catch {
    return null;
  }
}

function safeGetItem(key: string): string | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function safeSetItem(key: string, value: string) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // ignore
  }
}

function safeRemoveItem(key: string) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.removeItem(key);
  } catch {
    // ignore
  }
}

export function getTransactionCount(): number {
  const raw = safeGetItem(ONBOARDING_STORAGE_KEYS.transactionCount);
  const parsed = raw ? Number.parseInt(raw, 10) : 0;
  return Number.isFinite(parsed) ? parsed : 0;
}

export function setTransactionCount(count: number) {
  const next = Number.isFinite(count) ? Math.max(0, Math.floor(count)) : 0;
  safeSetItem(ONBOARDING_STORAGE_KEYS.transactionCount, String(next));
}

export function incrementTransactionCount(by = 1): number {
  const next = getTransactionCount() + by;
  setTransactionCount(next);
  return next;
}

export function getQuickStartState(): QuickStartState {
  const stored = safeJsonParse<Partial<QuickStartState>>(
    safeGetItem(ONBOARDING_STORAGE_KEYS.quickStart),
  );
  return {
    ...DEFAULT_QUICK_START,
    ...(stored ?? {}),
  };
}

export function setQuickStartState(
  patch: Partial<QuickStartState>,
): QuickStartState {
  const next = { ...getQuickStartState(), ...patch };
  safeSetItem(ONBOARDING_STORAGE_KEYS.quickStart, JSON.stringify(next));
  return next;
}

export function markQuickStartDone(step: QuickStartStep): QuickStartState {
  return setQuickStartState({ [step]: true });
}

export function isOnboardingDismissed(): boolean {
  return safeGetItem(ONBOARDING_STORAGE_KEYS.dismissed) === "1";
}

export function setOnboardingDismissed(value: boolean) {
  safeSetItem(ONBOARDING_STORAGE_KEYS.dismissed, value ? "1" : "0");
}

export function setOnboardingCompleted(value: boolean) {
  safeSetItem(ONBOARDING_STORAGE_KEYS.completed, value ? "1" : "0");
}

export function resetOnboarding() {
  safeRemoveItem(ONBOARDING_STORAGE_KEYS.dismissed);
  safeRemoveItem(ONBOARDING_STORAGE_KEYS.completed);
  safeRemoveItem(ONBOARDING_STORAGE_KEYS.quickStart);
}


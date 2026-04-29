'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import {
  AlertTriangle,
  ArrowDownToLine,
  ArrowUpFromLine,
  CheckCircle2,
  ChevronDown,
  Info,
  Loader2,
  X,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useLiquidity, SUPPORTED_TOKENS, type TokenBalance } from '@/hooks/useLiquidity';

// Vault contract address (testnet placeholder — replace with deployed address)
const VAULT_ADDRESS = 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5';

// Threshold above which a large-transaction warning is shown
const LARGE_TX_THRESHOLD = 1000;

type Tab = 'deposit' | 'withdraw';
type Step = 'form' | 'confirm' | 'submitting' | 'success';

interface LiquidityModalProps {
  isOpen: boolean;
  onClose: () => void;
  publicKey: string | null;
  /** Pre-select a tab when opening */
  defaultTab?: Tab;
}

export function LiquidityModal({
  isOpen,
  onClose,
  publicKey,
  defaultTab = 'deposit',
}: LiquidityModalProps) {
  const [tab, setTab] = useState<Tab>(defaultTab);
  const [selectedToken, setSelectedToken] = useState(SUPPORTED_TOKENS[0]);
  const [amount, setAmount] = useState('');
  const [step, setStep] = useState<Step>('form');
  const [txHash, setTxHash] = useState<string | null>(null);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [tokenMenuOpen, setTokenMenuOpen] = useState(false);

  const { balances, loadingBalances, balanceError, refreshBalances, validateAmount, executeOperation } =
    useLiquidity(publicKey);

  const overlayRef = useRef<HTMLDivElement>(null);
  const firstFocusRef = useRef<HTMLButtonElement>(null);

  // Load balances when modal opens
  useEffect(() => {
    if (isOpen && publicKey) {
      refreshBalances();
    }
  }, [isOpen, publicKey, refreshBalances]);

  // Reset state when modal opens/closes
  useEffect(() => {
    if (isOpen) {
      setTab(defaultTab);
      setAmount('');
      setStep('form');
      setTxHash(null);
      setSubmitError(null);
      setSelectedToken(SUPPORTED_TOKENS[0]);
      // Focus first interactive element
      setTimeout(() => firstFocusRef.current?.focus(), 50);
    }
  }, [isOpen, defaultTab]);

  // Trap focus inside modal
  useEffect(() => {
    if (!isOpen) return;
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  const selectedBalance = balances.find((b) => b.code === selectedToken.code);
  const availableBalance = selectedBalance ? parseFloat(selectedBalance.balance) : 0;

  const validationError = step === 'form' && amount ? validateAmount(amount, selectedToken) : null;
  const isLargeTx = parseFloat(amount) >= LARGE_TX_THRESHOLD;
  const canProceed = !!amount && !validationError && !loadingBalances;

  const handleMaxClick = () => {
    if (!selectedBalance) return;
    const max =
      selectedToken.code === 'XLM'
        ? Math.max(0, availableBalance - 1.5).toFixed(7)
        : availableBalance.toFixed(7);
    setAmount(max);
  };

  const handleTabChange = (newTab: Tab) => {
    setTab(newTab);
    setAmount('');
    setSubmitError(null);
  };

  const handleTokenSelect = (token: Omit<TokenBalance, 'balance'>) => {
    setSelectedToken(token);
    setAmount('');
    setTokenMenuOpen(false);
  };

  const handleProceed = () => {
    if (!canProceed) return;
    setStep('confirm');
  };

  const handleConfirm = async () => {
    setStep('submitting');
    setSubmitError(null);
    try {
      const hash = await executeOperation(tab, selectedToken, amount, VAULT_ADDRESS);
      setTxHash(hash);
      setStep('success');
      await refreshBalances();
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : 'Transaction failed. Please try again.');
      setStep('form');
    }
  };

  const handleClose = useCallback(() => {
    if (step === 'submitting') return; // prevent close during submission
    onClose();
  }, [step, onClose]);

  if (!isOpen) return null;

  return (
    <div
      ref={overlayRef}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="liquidity-modal-title"
      onClick={(e) => e.target === overlayRef.current && handleClose()}
    >
      <div className="bg-white dark:bg-gray-900 rounded-2xl shadow-xl w-full max-w-md border border-gray-200 dark:border-gray-800 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-gray-100 dark:border-gray-800">
          <h2
            id="liquidity-modal-title"
            className="text-lg font-semibold text-gray-900 dark:text-white"
          >
            Liquidity Management
          </h2>
          <button
            ref={firstFocusRef}
            onClick={handleClose}
            disabled={step === 'submitting'}
            aria-label="Close modal"
            className="p-2 rounded-full text-gray-400 hover:text-gray-600 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors disabled:opacity-40"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Success state */}
        {step === 'success' ? (
          <SuccessView
            tab={tab}
            amount={amount}
            token={selectedToken}
            txHash={txHash}
            onClose={handleClose}
            onAnother={() => {
              setStep('form');
              setAmount('');
              setTxHash(null);
            }}
          />
        ) : step === 'confirm' ? (
          <ConfirmView
            tab={tab}
            amount={amount}
            token={selectedToken}
            isLargeTx={isLargeTx}
            onBack={() => setStep('form')}
            onConfirm={handleConfirm}
          />
        ) : (
          /* Form state (also covers submitting overlay) */
          <div className="relative">
            {step === 'submitting' && (
              <div className="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm rounded-b-2xl">
                <Loader2 className="w-8 h-8 text-blue-600 animate-spin" />
                <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                  Submitting transaction…
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400 text-center px-6">
                  Please approve the request in your wallet. Do not close this window.
                </p>
              </div>
            )}

            {/* Tabs */}
            <div className="flex border-b border-gray-100 dark:border-gray-800">
              {(['deposit', 'withdraw'] as Tab[]).map((t) => (
                <button
                  key={t}
                  onClick={() => handleTabChange(t)}
                  aria-selected={tab === t}
                  role="tab"
                  className={`flex-1 flex items-center justify-center gap-2 py-3 text-sm font-medium transition-colors ${
                    tab === t
                      ? 'text-blue-600 border-b-2 border-blue-600 dark:text-blue-400 dark:border-blue-400'
                      : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                  }`}
                >
                  {t === 'deposit' ? (
                    <ArrowDownToLine className="w-4 h-4" />
                  ) : (
                    <ArrowUpFromLine className="w-4 h-4" />
                  )}
                  {t.charAt(0).toUpperCase() + t.slice(1)}
                </button>
              ))}
            </div>

            <div className="p-5 space-y-4">
              {/* Wallet not connected */}
              {!publicKey && (
                <div className="flex items-start gap-2 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 px-3 py-2.5 text-sm text-amber-700 dark:text-amber-300">
                  <Info className="w-4 h-4 mt-0.5 shrink-0" />
                  Connect your wallet to manage liquidity.
                </div>
              )}

              {/* Token selector */}
              <div>
                <label className="block text-xs font-medium text-gray-500 dark:text-gray-400 mb-1.5">
                  Token
                </label>
                <div className="relative">
                  <button
                    type="button"
                    onClick={() => setTokenMenuOpen((o) => !o)}
                    aria-haspopup="listbox"
                    aria-expanded={tokenMenuOpen}
                    className="w-full flex items-center justify-between gap-2 rounded-xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100 hover:border-blue-400 transition-colors"
                  >
                    <span className="flex items-center gap-2">
                      <TokenIcon code={selectedToken.code} />
                      {selectedToken.code}
                    </span>
                    <ChevronDown className={`w-4 h-4 text-gray-400 transition-transform ${tokenMenuOpen ? 'rotate-180' : ''}`} />
                  </button>

                  {tokenMenuOpen && (
                    <ul
                      role="listbox"
                      aria-label="Select token"
                      className="absolute z-20 mt-1 w-full rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 shadow-lg overflow-hidden"
                    >
                      {SUPPORTED_TOKENS.map((token) => (
                        <li key={token.code}>
                          <button
                            type="button"
                            role="option"
                            aria-selected={selectedToken.code === token.code}
                            onClick={() => handleTokenSelect(token)}
                            className="w-full flex items-center gap-3 px-4 py-3 text-sm text-gray-900 dark:text-gray-100 hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors"
                          >
                            <TokenIcon code={token.code} />
                            <span className="font-medium">{token.code}</span>
                            {selectedToken.code === token.code && (
                              <CheckCircle2 className="w-4 h-4 text-blue-600 ml-auto" />
                            )}
                          </button>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              </div>

              {/* Amount input */}
              <div>
                <div className="flex items-center justify-between mb-1.5">
                  <label
                    htmlFor="liquidity-amount"
                    className="text-xs font-medium text-gray-500 dark:text-gray-400"
                  >
                    Amount
                  </label>
                  <span className="text-xs text-gray-500 dark:text-gray-400">
                    {loadingBalances ? (
                      <span className="inline-flex items-center gap-1">
                        <Loader2 className="w-3 h-3 animate-spin" /> Loading…
                      </span>
                    ) : balanceError ? (
                      <span className="text-red-500">{balanceError}</span>
                    ) : (
                      <>
                        Available:{' '}
                        <button
                          type="button"
                          onClick={handleMaxClick}
                          className="font-semibold text-blue-600 dark:text-blue-400 hover:underline"
                          aria-label={`Set max amount: ${availableBalance} ${selectedToken.code}`}
                        >
                          {availableBalance.toLocaleString(undefined, { maximumFractionDigits: 4 })}{' '}
                          {selectedToken.code}
                        </button>
                      </>
                    )}
                  </span>
                </div>
                <div className="relative">
                  <input
                    id="liquidity-amount"
                    type="number"
                    inputMode="decimal"
                    min="0"
                    step="any"
                    value={amount}
                    onChange={(e) => {
                      setAmount(e.target.value);
                      setSubmitError(null);
                    }}
                    placeholder="0.00"
                    disabled={!publicKey || step === 'submitting'}
                    aria-describedby={validationError ? 'amount-error' : undefined}
                    aria-invalid={!!validationError}
                    className={`w-full rounded-xl border px-4 py-3 pr-16 text-base font-medium text-gray-900 dark:text-gray-100 bg-white dark:bg-gray-800 outline-none transition-colors placeholder:text-gray-400 disabled:opacity-50 ${
                      validationError
                        ? 'border-red-400 focus:border-red-500 focus:ring-2 focus:ring-red-100 dark:focus:ring-red-900/30'
                        : 'border-gray-200 dark:border-gray-700 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 dark:focus:ring-blue-900/30'
                    }`}
                  />
                  <span className="absolute right-4 top-1/2 -translate-y-1/2 text-sm font-medium text-gray-500 dark:text-gray-400 pointer-events-none">
                    {selectedToken.code}
                  </span>
                </div>

                {validationError && (
                  <p
                    id="amount-error"
                    role="alert"
                    className="mt-1.5 flex items-center gap-1.5 text-xs text-red-600 dark:text-red-400"
                  >
                    <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
                    {validationError}
                  </p>
                )}

                {submitError && (
                  <p
                    role="alert"
                    className="mt-1.5 flex items-center gap-1.5 text-xs text-red-600 dark:text-red-400"
                  >
                    <AlertTriangle className="w-3.5 h-3.5 shrink-0" />
                    {submitError}
                  </p>
                )}
              </div>

              {/* Finality warning */}
              <div className="flex items-start gap-2 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-100 dark:border-blue-800 px-3 py-2.5 text-xs text-blue-700 dark:text-blue-300">
                <Info className="w-3.5 h-3.5 mt-0.5 shrink-0" />
                Stellar transactions are final and irreversible. Verify all details before submitting.
              </div>

              {/* Action button */}
              <Button
                variant="primary"
                size="lg"
                fullWidth
                disabled={!canProceed || !publicKey}
                onClick={handleProceed}
                aria-label={`${tab === 'deposit' ? 'Deposit' : 'Withdraw'} ${amount || '0'} ${selectedToken.code}`}
              >
                {tab === 'deposit' ? (
                  <ArrowDownToLine className="w-4 h-4 mr-2" />
                ) : (
                  <ArrowUpFromLine className="w-4 h-4 mr-2" />
                )}
                {tab === 'deposit' ? 'Deposit' : 'Withdraw'}
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// ── Sub-views ────────────────────────────────────────────────────────────────

function ConfirmView({
  tab,
  amount,
  token,
  isLargeTx,
  onBack,
  onConfirm,
}: {
  tab: Tab;
  amount: string;
  token: Omit<TokenBalance, 'balance'>;
  isLargeTx: boolean;
  onBack: () => void;
  onConfirm: () => void;
}) {
  return (
    <div className="p-5 space-y-4">
      <p className="text-sm text-gray-600 dark:text-gray-400">
        Review your transaction before submitting.
      </p>

      <dl className="rounded-xl bg-gray-50 dark:bg-gray-800 divide-y divide-gray-100 dark:divide-gray-700 text-sm">
        <Row label="Action" value={tab === 'deposit' ? 'Deposit' : 'Withdraw'} />
        <Row label="Token" value={token.code} />
        <Row label="Amount" value={`${amount} ${token.code}`} highlight />
        <Row label="Network" value="Stellar Testnet" />
        <Row label="Estimated fee" value="~0.00001 XLM" />
      </dl>

      {isLargeTx && (
        <div
          role="alert"
          className="flex items-start gap-2 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 px-3 py-2.5 text-sm text-amber-700 dark:text-amber-300"
        >
          <AlertTriangle className="w-4 h-4 mt-0.5 shrink-0" />
          <span>
            <strong>Large transaction:</strong> You are about to {tab}{' '}
            {amount} {token.code}. Please confirm this is intentional.
          </span>
        </div>
      )}

      <div className="flex gap-3">
        <Button variant="outline" size="md" fullWidth onClick={onBack}>
          Back
        </Button>
        <Button variant="primary" size="md" fullWidth onClick={onConfirm}>
          Confirm &amp; Sign
        </Button>
      </div>
    </div>
  );
}

function SuccessView({
  tab,
  amount,
  token,
  txHash,
  onClose,
  onAnother,
}: {
  tab: Tab;
  amount: string;
  token: Omit<TokenBalance, 'balance'>;
  txHash: string | null;
  onClose: () => void;
  onAnother: () => void;
}) {
  return (
    <div className="p-5 flex flex-col items-center gap-4 text-center">
      <div className="w-14 h-14 rounded-full bg-emerald-100 dark:bg-emerald-900/30 flex items-center justify-center">
        <CheckCircle2 className="w-8 h-8 text-emerald-600 dark:text-emerald-400" />
      </div>
      <div>
        <p className="text-base font-semibold text-gray-900 dark:text-white">
          {tab === 'deposit' ? 'Deposit' : 'Withdrawal'} successful!
        </p>
        <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
          {amount} {token.code} {tab === 'deposit' ? 'deposited into' : 'withdrawn from'} the vault.
        </p>
      </div>

      {txHash && (
        <a
          href={`https://stellar.expert/explorer/testnet/tx/${txHash}`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-xs text-blue-600 dark:text-blue-400 hover:underline break-all"
        >
          View on StellarExpert ↗
        </a>
      )}

      <div className="flex gap-3 w-full">
        <Button variant="outline" size="md" fullWidth onClick={onAnother}>
          New transaction
        </Button>
        <Button variant="primary" size="md" fullWidth onClick={onClose}>
          Done
        </Button>
      </div>
    </div>
  );
}

function Row({
  label,
  value,
  highlight,
}: {
  label: string;
  value: string;
  highlight?: boolean;
}) {
  return (
    <div className="flex items-center justify-between px-4 py-2.5">
      <dt className="text-gray-500 dark:text-gray-400">{label}</dt>
      <dd className={`font-medium ${highlight ? 'text-blue-600 dark:text-blue-400' : 'text-gray-900 dark:text-gray-100'}`}>
        {value}
      </dd>
    </div>
  );
}

function TokenIcon({ code }: { code: string }) {
  const colors: Record<string, string> = {
    XLM: 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300',
    USDC: 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300',
  };
  return (
    <span
      className={`inline-flex items-center justify-center w-6 h-6 rounded-full text-xs font-bold ${colors[code] ?? 'bg-gray-100 text-gray-600'}`}
      aria-hidden="true"
    >
      {code.slice(0, 1)}
    </span>
  );
}

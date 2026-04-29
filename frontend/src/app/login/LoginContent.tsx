"use client";

import { useEffect, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { ConnectButton } from '@/components/auth/ConnectButton';
import { useWalletAuth } from '@/hooks/useWalletAuth';
import { Loader2 } from 'lucide-react';
import { NETWORKS } from '@/utils/stellar';

const formatBalance = (balances: Array<{ asset_type: string; balance: string }>) => {
  const native = balances.find((item) => item.asset_type === 'native');
  if (native) return `${parseFloat(native.balance).toFixed(2)} XLM`;
  return `${balances[0]?.balance ?? '0.00'}`;
};

export function LoginContent() {
  const { isConnected, isCheckingSession, publicKey, walletType } = useWalletAuth();
  const router = useRouter();
  const searchParams = useSearchParams();
  const [balance, setBalance] = useState<string | null>(null);
  const [balanceError, setBalanceError] = useState<string | null>(null);

  useEffect(() => {
    if (!isCheckingSession && isConnected) {
      const from = searchParams.get('from') ?? '/dashboard';
      router.replace(from);
    }
  }, [isConnected, isCheckingSession, router, searchParams]);

  useEffect(() => {
    async function fetchBalance() {
      if (!publicKey) return;
      setBalanceError(null);
      setBalance(null);

      try {
        const res = await fetch(`${NETWORKS.testnet.horizonUrl}/accounts/${publicKey}`);
        if (!res.ok) {
          throw new Error('Unable to query account balances.');
        }
        const data = await res.json();
        setBalance(formatBalance(data.balances ?? []));
      } catch {
        setBalanceError('Balance unavailable.');
      }
    }

    if (isConnected && publicKey) {
      fetchBalance();
    }
  }, [isConnected, publicKey]);

  if (isCheckingSession) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <Loader2 className="w-6 h-6 animate-spin text-gray-400" />
      </div>
    );
  }

  return (
    <div className="min-h-screen flex flex-col items-center justify-center p-4 bg-gradient-to-b from-white to-gray-50 dark:from-black dark:to-gray-900">
      <div className="w-full max-w-md text-center space-y-8">
        <div className="space-y-3">
          <div>
            <h1 className="text-4xl font-bold tracking-tight text-gray-900 dark:text-white">
              Welcome to StelloVault
            </h1>
            <p className="mt-2 text-gray-500 dark:text-gray-400">
              Securely connect your Stellar wallet and manage collateral tokenization with a single browser flow.
            </p>
          </div>
          <div className="rounded-3xl border border-gray-200 bg-white p-4 text-left text-sm text-gray-600 shadow-sm dark:border-gray-700 dark:bg-gray-950 dark:text-gray-300">
            <p className="font-semibold text-gray-900 dark:text-white">New to Stellar wallets?</p>
            <ol className="mt-3 space-y-2 pl-5 text-gray-600 dark:text-gray-400">
              <li>1. Install Freighter or choose WalletConnect from the connect menu.</li>
              <li>2. Approve the connection request in your wallet.</li>
              <li>3. Return to StelloVault to view your account summary and dashboard.</li>
            </ol>
          </div>
        </div>

        <div className="p-8 bg-white dark:bg-gray-900 rounded-2xl shadow-xl border border-gray-100 dark:border-gray-800 flex flex-col items-center justify-center min-h-[200px]">
          <ConnectButton />

          {isConnected && publicKey && (
            <div className="mt-6 w-full rounded-3xl bg-gray-50 p-4 text-left text-sm text-gray-700 shadow-sm dark:bg-gray-950 dark:text-gray-200">
              <p className="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Account summary
              </p>
              <div className="mt-4 grid gap-3 sm:grid-cols-2">
                <div className="rounded-2xl border border-gray-200 bg-white p-3 dark:border-gray-700 dark:bg-gray-900">
                  <p className="text-xs text-gray-500 dark:text-gray-400">Wallet</p>
                  <p className="mt-1 text-sm font-semibold text-gray-900 dark:text-white">
                    {walletType ? `${walletType.charAt(0).toUpperCase()}${walletType.slice(1)}` : 'Stellar wallet'}
                  </p>
                </div>
                <div className="rounded-2xl border border-gray-200 bg-white p-3 dark:border-gray-700 dark:bg-gray-900">
                  <p className="text-xs text-gray-500 dark:text-gray-400">Address</p>
                  <p className="mt-1 break-all text-sm font-mono text-gray-900 dark:text-white">{publicKey}</p>
                </div>
                <div className="sm:col-span-2 rounded-2xl border border-gray-200 bg-white p-3 dark:border-gray-700 dark:bg-gray-900">
                  <p className="text-xs text-gray-500 dark:text-gray-400">Balance</p>
                  <p className="mt-1 text-sm font-semibold text-gray-900 dark:text-white">
                    {balance ?? balanceError ?? 'Loading balance...'}
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>

        <div className="text-xs text-gray-400">
          Non-custodial &amp; secure. We support Freighter and WalletConnect today, with support for additional Stellar/Soroban wallets planned.
        </div>
      </div>
    </div>
  );
}

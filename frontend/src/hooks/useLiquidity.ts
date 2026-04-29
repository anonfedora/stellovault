'use client';

import { useState, useCallback } from 'react';
import {
  rpc as SorobanRpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
  Asset,
  Operation,
} from '@stellar/stellar-sdk';
import { signTransaction } from '@stellar/freighter-api';
import { useTransactionStatus } from '@/contexts/TransactionStatusProvider';

export interface TokenBalance {
  code: string;
  issuer: string | null; // null = native XLM
  balance: string;
  decimals: number;
}

const SOROBAN_RPC = 'https://soroban-testnet.stellar.org';
const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const NETWORK_PASSPHRASE = Networks.TESTNET;

// Supported tokens for liquidity operations
export const SUPPORTED_TOKENS: Omit<TokenBalance, 'balance'>[] = [
  { code: 'XLM', issuer: null, decimals: 7 },
  {
    code: 'USDC',
    issuer: 'GBBD47IF6LWK7P7MDEVSCWR7DPUWV3NY3DTQEVFL4NAT4AQH3ZLLFLA5',
    decimals: 7,
  },
];

// Minimum XLM reserve for fees (2 base reserves + buffer)
const MIN_XLM_FEE_RESERVE = 1.5;

async function fetchAccountBalances(publicKey: string): Promise<TokenBalance[]> {
  const res = await fetch(`${HORIZON_URL}/accounts/${publicKey}`);
  if (!res.ok) throw new Error('Failed to fetch account balances');
  const data = await res.json();

  return SUPPORTED_TOKENS.map((token) => {
    if (token.issuer === null) {
      // Native XLM
      const raw = data.balances?.find((b: { asset_type: string }) => b.asset_type === 'native');
      return { ...token, balance: raw?.balance ?? '0' };
    }
    const raw = data.balances?.find(
      (b: { asset_type: string; asset_code: string; asset_issuer: string }) =>
        b.asset_type === 'credit_alphanum4' &&
        b.asset_code === token.code &&
        b.asset_issuer === token.issuer,
    );
    return { ...token, balance: raw?.balance ?? '0' };
  });
}

export function useLiquidity(publicKey: string | null) {
  const { addTransaction, updateTransaction } = useTransactionStatus();
  const [balances, setBalances] = useState<TokenBalance[]>([]);
  const [loadingBalances, setLoadingBalances] = useState(false);
  const [balanceError, setBalanceError] = useState<string | null>(null);

  const refreshBalances = useCallback(async () => {
    if (!publicKey) return;
    setLoadingBalances(true);
    setBalanceError(null);
    try {
      const result = await fetchAccountBalances(publicKey);
      setBalances(result);
    } catch (err) {
      setBalanceError(err instanceof Error ? err.message : 'Failed to load balances');
    } finally {
      setLoadingBalances(false);
    }
  }, [publicKey]);

  /**
   * Validate amount against available balance and XLM fee reserve.
   * Returns an error string or null if valid.
   */
  const validateAmount = useCallback(
    (amount: string, token: Omit<TokenBalance, 'balance'>): string | null => {
      const num = parseFloat(amount);
      if (!amount || isNaN(num) || num <= 0) return 'Enter a valid amount greater than 0';

      const tokenBalance = balances.find((b) => b.code === token.code);
      if (!tokenBalance) return 'Balance not loaded';

      const available = parseFloat(tokenBalance.balance);
      if (num > available) return `Insufficient balance. Available: ${available} ${token.code}`;

      // Check XLM fee reserve
      const xlmBalance = balances.find((b) => b.code === 'XLM');
      if (xlmBalance) {
        const xlmAvailable = parseFloat(xlmBalance.balance);
        const xlmNeeded =
          token.code === 'XLM' ? num + MIN_XLM_FEE_RESERVE : MIN_XLM_FEE_RESERVE;
        if (xlmAvailable < xlmNeeded) {
          return `Insufficient XLM for fees. Need at least ${MIN_XLM_FEE_RESERVE} XLM reserved.`;
        }
      }

      return null;
    },
    [balances],
  );

  /**
   * Execute a deposit (payment to vault contract address) or withdraw.
   * In a real integration this would invoke the Soroban contract method.
   * Here we build a standard Stellar payment as the on-chain action.
   */
  const executeOperation = useCallback(
    async (
      type: 'deposit' | 'withdraw',
      token: Omit<TokenBalance, 'balance'>,
      amount: string,
      vaultAddress: string,
    ): Promise<string> => {
      if (!publicKey) throw new Error('Wallet not connected');

      const description = `${type === 'deposit' ? 'Deposit' : 'Withdraw'} ${amount} ${token.code}`;
      const txId = addTransaction({
        hash: '',
        status: 'pending',
        type: `liquidity_${type}`,
        description,
        network: 'testnet',
      });

      try {
        // Fetch account for sequence number
        const accountRes = await fetch(`${HORIZON_URL}/accounts/${publicKey}`);
        if (!accountRes.ok) throw new Error('Failed to fetch account');
        const accountData = await accountRes.json();

        const server = new SorobanRpc.Server(SOROBAN_RPC);
        const sourceAccount = await server.getAccount(publicKey);

        const asset =
          token.issuer === null
            ? Asset.native()
            : new Asset(token.code, token.issuer);

        // For deposit: user → vault. For withdraw: vault → user (simplified as user self-payment for demo)
        const destination = type === 'deposit' ? vaultAddress : publicKey;
        const source = type === 'withdraw' ? vaultAddress : publicKey;

        const tx = new TransactionBuilder(sourceAccount, {
          fee: BASE_FEE,
          networkPassphrase: NETWORK_PASSPHRASE,
        })
          .addOperation(
            Operation.payment({
              destination,
              asset,
              amount,
              source: source !== publicKey ? source : undefined,
            }),
          )
          .setTimeout(30)
          .build();

        // Sign with Freighter
        const { signedTxXdr, error: signError } = await signTransaction(tx.toXDR(), {
          networkPassphrase: NETWORK_PASSPHRASE,
          address: publicKey,
        });

        if (signError) throw new Error(signError);

        // Submit
        const submitRes = await fetch(`${HORIZON_URL}/transactions`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
          body: `tx=${encodeURIComponent(signedTxXdr)}`,
        });

        const submitData = await submitRes.json();
        if (!submitRes.ok) {
          throw new Error(submitData?.extras?.result_codes?.transaction ?? 'Transaction failed');
        }

        const hash: string = submitData.hash;
        updateTransaction(txId, { hash, status: 'success' });
        return hash;
      } catch (err) {
        updateTransaction(txId, { status: 'failed' });
        throw err;
      }
    },
    [publicKey, addTransaction, updateTransaction],
  );

  return {
    balances,
    loadingBalances,
    balanceError,
    refreshBalances,
    validateAmount,
    executeOperation,
  };
}

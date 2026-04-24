import { useState, useEffect, useCallback } from 'react';
import { isAllowed, setAllowed, getAddress, signMessage } from '@stellar/freighter-api';
import { useRouter } from 'next/navigation';
import { markQuickStartDone } from '@/utils/onboarding';

export type WalletType = 'freighter' | 'walletconnect' | null;

interface WalletAuth {
  isConnected: boolean;
  isConnecting: boolean;
  isCheckingSession: boolean;
  publicKey: string | null;
  walletType: WalletType;
  connect: (type?: WalletType) => Promise<string | null>;
  login: (key?: string) => Promise<void>;
  logout: () => Promise<void>;
  error: string | null;
  clearError: () => void;
}

async function silentRefresh(): Promise<boolean> {
  try {
    const res = await fetch('/api/v1/auth/refresh', { method: 'POST' });
    return res.ok;
  } catch {
    return false;
  }
}

export function useWalletAuth(): WalletAuth {
  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [isCheckingSession, setIsCheckingSession] = useState(true);
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [walletType, setWalletType] = useState<WalletType>(null);
  const [error, setError] = useState<string | null>(null);
  const router = useRouter();

  const clearError = useCallback(() => setError(null), []);

  // Restore session on mount — check Freighter + cookie validity
  useEffect(() => {
    async function restoreSession() {
      try {
        const result = await isAllowed();
        if (result?.isAllowed) {
          const { address, error: addrErr } = await getAddress();
          if (address && !addrErr) {
            setPublicKey(address);
            setWalletType('freighter');
            setIsConnected(true);
            markQuickStartDone('connectWallet');
          }
        }
      } catch {
        // Freighter not installed or unavailable — that's fine
      } finally {
        setIsCheckingSession(false);
      }
    }
    restoreSession();
  }, []);

  const connect = useCallback(async (type: WalletType = 'freighter'): Promise<string | null> => {
    setIsConnecting(true);
    setError(null);

    try {
      if (type === 'freighter') {
        const result = await setAllowed();
        if (!result?.isAllowed) {
          setError('Connection refused. Please approve the request in Freighter.');
          return null;
        }
        const { address, error: addrErr } = await getAddress();
        if (addrErr || !address) {
          setError('Could not retrieve wallet address. Is Freighter unlocked?');
          return null;
        }
        setPublicKey(address);
        setWalletType('freighter');
        setIsConnected(true);
        markQuickStartDone('connectWallet');
        return address;
      }

      // WalletConnect — placeholder
      setError('WalletConnect is coming soon.');
      return null;
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      if (msg.toLowerCase().includes('freighter') || msg.toLowerCase().includes('extension')) {
        setError('Freighter extension not found. Please install it first.');
      } else {
        setError('Failed to connect wallet. Please try again.');
      }
      return null;
    } finally {
      setIsConnecting(false);
    }
  }, []);

  const login = useCallback(async (key?: string) => {
    const pk = key ?? publicKey;
    if (!pk) {
      setError('No wallet connected.');
      return;
    }

    setIsConnecting(true);
    setError(null);

    try {
      // 1. Request challenge
      const challengeRes = await fetch('/api/v1/auth/challenge', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ publicKey: pk }),
      });

      if (!challengeRes.ok) {
        const body = await challengeRes.json().catch(() => ({}));
        throw new Error(body.error ?? 'Failed to get authentication challenge.');
      }

      const { nonce } = await challengeRes.json();

      // 2. Sign nonce with Freighter
      const signResult = await signMessage(nonce, { address: pk });
      if (signResult.error) {
        throw new Error('Signing was rejected or failed. Please try again.');
      }

      const signedMessage =
        typeof signResult.signedMessage === 'string'
          ? signResult.signedMessage
          : Buffer.from(signResult.signedMessage as Uint8Array).toString('base64');

      // 3. Verify signature → sets httpOnly cookies
      const verifyRes = await fetch('/api/v1/auth/verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          publicKey: pk,
          signedMessage,
          signerPublicKey: signResult.signerAddress,
        }),
      });

      if (!verifyRes.ok) {
        const body = await verifyRes.json().catch(() => ({}));
        throw new Error(body.error ?? 'Signature verification failed.');
      }

      router.push('/dashboard');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : 'Login failed. Please try again.';
      setError(msg);
    } finally {
      setIsConnecting(false);
    }
  }, [publicKey, router]);

  const logout = useCallback(async () => {
    try {
      await fetch('/api/v1/auth/logout', { method: 'POST' });
    } catch {
      // best-effort
    }
    setIsConnected(false);
    setPublicKey(null);
    setWalletType(null);
    router.push('/login');
  }, [router]);

  // Silent token refresh — runs once after session check
  useEffect(() => {
    if (isCheckingSession) return;
    silentRefresh().catch(() => {});
  }, [isCheckingSession]);

  return {
    isConnected,
    isConnecting,
    isCheckingSession,
    publicKey,
    walletType,
    connect,
    login,
    logout,
    error,
    clearError,
  };
}

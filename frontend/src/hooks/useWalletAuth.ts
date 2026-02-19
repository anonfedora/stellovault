import { useState, useEffect } from 'react';
import { isAllowed, setAllowed, getAddress, signMessage } from '@stellar/freighter-api';

interface WalletAuth {
    isConnected: boolean;
    isConnecting: boolean;
    publicKey: string | null;
    connect: () => Promise<string | null>;
    login: (key?: string) => Promise<void>;
    logout: () => Promise<void>;
    error: string | null;
}

export function useWalletAuth(): WalletAuth {
    const [isConnected, setIsConnected] = useState(false);
    const [isConnecting, setIsConnecting] = useState(false);
    const [publicKey, setPublicKey] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        // Check if already allowed/connected on mount
        async function checkConnection() {
            if (await isAllowed()) {
                const { address } = await getAddress();
                if (address) {
                    setIsConnected(true);
                    setPublicKey(address);
                }
            }
        }
        checkConnection();
    }, []);

    const connect = async () => {
        setIsConnecting(true);
        setError(null);
        let key: string | null = null;
        try {
            const allowed = await setAllowed();
            if (allowed) {
                const { address } = await getAddress();
                if (address) {
                    setIsConnected(true);
                    setPublicKey(address);
                    key = address;
                }
            } else {
                setError('User refused connection');
            }
        } catch (err) {
            setError('Failed to connect wallet');
            console.error(err);
        } finally {
            setIsConnecting(false);
        }
        return key;
    };

    const login = async (key?: string) => {
        const pk = key || publicKey;
        if (!pk) {
            setError('Wallet not connected');
            return;
        }
        setIsConnecting(true);
        setError(null);

        try {
            // 1. Get Challenge
            const challengeRes = await fetch('/api/v1/auth/challenge', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ publicKey: pk }),
            });

            if (!challengeRes.ok) throw new Error('Failed to get challenge');
            const { nonce } = await challengeRes.json();

            // 2. Sign Message
            const signedMessage = await signMessage(nonce);

            // 3. Verify
            const verifyRes = await fetch('/api/v1/auth/verify', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ publicKey: pk, signedMessage }),
            });

            if (!verifyRes.ok) throw new Error('Verification failed');

            // Success - redirect or whatever
            window.location.href = '/dashboard';

        } catch (err: any) {
            setError(err.message || 'Login failed');
            console.error(err);
        } finally {
            setIsConnecting(false);
        }
    };

    const logout = async () => {
        // Call API to clear cookies if needed, or just clear local state
        // Ideally call an endpoint to clear HTTP-only cookies
        setIsConnected(false);
        setPublicKey(null);
        await fetch('/api/v1/auth/logout', { method: 'POST' }); // We might need this endpoint
        window.location.href = '/login';
    };

    return {
        isConnected,
        isConnecting,
        publicKey,
        connect,
        login,
        logout,
        error,
    };
}

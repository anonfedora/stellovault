import { useState, useEffect } from 'react';
import { isAllowed, setAllowed, getAddress, signMessage } from '@stellar/freighter-api';
import { useRouter } from 'next/navigation';

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
    const router = useRouter();

    useEffect(() => {
        // Check if already allowed/connected on mount
        async function checkConnection() {
            try {
                const result = await isAllowed();
                if (result && result.isAllowed) {
                    const { address, error: addressError } = await getAddress();
                    if (address && !addressError) {
                        setIsConnected(true);
                        setPublicKey(address);
                    }
                }
            } catch (err) {
                console.error('Failed to check wallet connection:', err);
            }
        }
        checkConnection();
    }, []);

    const connect = async () => {
        setIsConnecting(true);
        setError(null);
        let key: string | null = null;
        try {
            const result = await setAllowed();
            if (result && result.isAllowed) {
                const { address, error: addressError } = await getAddress();
                if (address && !addressError) {
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
            const result = await signMessage(nonce);
            if (result.error) throw new Error(result.error);

            const signedMessageStr = typeof result.signedMessage === 'string'
                ? result.signedMessage
                : Buffer.from(result.signedMessage as any).toString('base64');
            const signerPublicKey = result.signerAddress;

            // 3. Verify
            const verifyRes = await fetch('/api/v1/auth/verify', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ publicKey: pk, signedMessage: signedMessageStr, signerPublicKey }),
            });

            if (!verifyRes.ok) throw new Error('Verification failed');

            // Success - redirect
            router.push('/dashboard');

        } catch (err: any) {
            setError(err.message || 'Login failed');
            console.error(err);
        } finally {
            setIsConnecting(false);
        }
    };

    const logout = async () => {
        try {
            // Call API to clear cookies first
            await fetch('/api/v1/auth/logout', { method: 'POST' });
        } catch (err) {
            console.error('Logout API call failed:', err);
        }

        setIsConnected(false);
        setPublicKey(null);
        router.push('/login');
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

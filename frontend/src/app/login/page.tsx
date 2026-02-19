"use client";

import { useEffect } from 'react';
import { ConnectButton } from '@/components/auth/ConnectButton';
import { useRouter } from 'next/navigation';
import { useWalletAuth } from '@/hooks/useWalletAuth';

export default function LoginPage() {
    const router = useRouter();
    const { isConnected, publicKey } = useWalletAuth();

    useEffect(() => {
        // If we have a public key, we might be logged in.
        // The `useWalletAuth` doesn't strictly track "isLoggedIn" via cookie check on client (yet).
        // But if the user completes the flow, `login` function redirects to dashboard.
        // Here we can just listen for `publicKey` changes or similar.
        // For now, let's rely on the `login` function's redirect.
    }, [isConnected, publicKey, router]);

    return (
        <div className="min-h-screen flex flex-col items-center justify-center p-4 bg-gradient-to-b from-white to-gray-50 dark:from-black dark:to-gray-900">
            <div className="w-full max-w-md text-center space-y-8">
                <div className="space-y-2">
                    <h1 className="text-4xl font-bold tracking-tight text-gray-900 dark:text-white">
                        Welcome Back
                    </h1>
                    <p className="text-gray-500 dark:text-gray-400">
                        Connect your wallet to access your dashboard securely.
                    </p>
                </div>

                <div className="p-8 bg-white dark:bg-gray-900 rounded-2xl shadow-xl border border-gray-100 dark:border-gray-800 flex flex-col items-center justify-center min-h-[200px]">
                    <ConnectButton />

                    <p className="mt-6 text-xs text-gray-400 max-w-xs">
                        We support Freighter and WalletConnect. Non-custodial & secure.
                    </p>
                </div>
            </div>
        </div>
    );
}

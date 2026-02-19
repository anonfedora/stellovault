import { useState } from 'react';
import { useWalletAuth } from '@/hooks/useWalletAuth';
import { WalletPickerModal } from './WalletPickerModal';
import { Loader2, Wallet } from 'lucide-react';
import { twMerge } from 'tailwind-merge';

export function ConnectButton() {
    const { isConnected, isConnecting, publicKey, login, logout, connect } = useWalletAuth();
    const [isModalOpen, setIsModalOpen] = useState(false);

    const handleConnect = async () => {
        // In this flow, we connect then immediately try to login (sign message)
        // The modal's onConnect just triggers the login process which includes `connect` internally if needed
        // But `useWalletAuth` separates `connect` (get key) and `login` (sign/verify)
        // Let's assume the modal just initiates the connection/login flow found in useWalletAuth

        // Actually, `useWalletAuth` `connect` just gets the public key. `login` does the challenge.
        // We should probably chain them effectively.

        const key = await connect();
        if (key) {
            await login(key);
        }
    };

    // If we are already connected (have public key), we show that.
    // But maybe we are not "logged in" (no JWT).
    // For simplicity, we assume `isConnected` means we have the public key in state.
    // The backend verification (login) is needed for protected routes.
    // If `publicKey` is present but cookie is missing, requests will fail.
    // We might want to auto-login if publicKey is present but we get 401.

    if (isConnected && publicKey) {
        return (
            <div className="flex items-center gap-2">
                <button
                    onClick={logout}
                    className="flex items-center gap-2 px-4 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 rounded-full text-sm font-medium transition-colors"
                >
                    <div className="w-2 h-2 rounded-full bg-green-500" />
                    <span>{publicKey.slice(0, 4)}...{publicKey.slice(-4)}</span>
                </button>
            </div>
        );
    }

    return (
        <>
            <button
                onClick={() => setIsModalOpen(true)}
                disabled={isConnecting}
                className={twMerge(
                    "flex items-center gap-2 px-6 py-2.5 bg-black dark:bg-white text-white dark:text-black rounded-full font-medium transition-all hover:opacity-90 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed",
                    isConnecting && "opacity-75"
                )}
            >
                {isConnecting ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                    <Wallet className="w-4 h-4" />
                )}
                <span>{isConnecting ? 'Connecting...' : 'Connect Wallet'}</span>
            </button>

            <WalletPickerModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                onConnect={handleConnect}
            />
        </>
    );
}

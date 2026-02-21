import { useState } from 'react';
import { useWalletAuth } from '@/hooks/useWalletAuth';
import { WalletPickerModal } from './WalletPickerModal';
import { Loader2, Wallet } from 'lucide-react';
import { twMerge } from 'tailwind-merge';

export function ConnectButton() {
    const { isConnected, isConnecting, publicKey, login, logout, connect } = useWalletAuth();
    const [isModalOpen, setIsModalOpen] = useState(false);

    const handleConnect = async () => {
        // Obtain public key then perform challenge-response login


        const key = await connect();
        if (key) {
            await login(key);
        }
    };

    // Display truncated address if connected. Login is required for protected routes.


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

import { useEffect, useState } from 'react';
import { isAllowed } from '@stellar/freighter-api';
import { X } from 'lucide-react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

interface WalletPickerModalProps {
    isOpen: boolean;
    onClose: () => void;
    onConnect: () => void;
}

export function WalletPickerModal({ isOpen, onClose, onConnect }: WalletPickerModalProps) {
    const [hasFreighter, setHasFreighter] = useState(false);

    useEffect(() => {
        // Check if Freighter is installed
        // The freighter-api doesn't expose a direct "isInstalled" check easily without try/catch or checking window
        // But we can try to see if the extension is present in window.
        // However, usually we just try to connect.
        const checkFreighter = async () => {
            // Simple check if the extension object exists
            // @ts-ignore
            if (window.freighter) {
                setHasFreighter(true);
            }
        };
        checkFreighter();
    }, []);

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
            <div className="bg-white dark:bg-gray-900 rounded-2xl shadow-xl w-full max-w-md overflow-hidden border border-gray-200 dark:border-gray-800">
                <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-800">
                    <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Connect Wallet</h2>
                    <button
                        onClick={onClose}
                        className="p-2 text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-full transition-colors"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                <div className="p-4 space-y-3">
                    <button
                        onClick={() => {
                            if (hasFreighter) {
                                onConnect();
                                onClose();
                            } else {
                                window.open('https://www.freighter.app/', '_blank');
                            }
                        }}
                        className={twMerge(
                            "w-full flex items-center p-4 rounded-xl border-2 transition-all duration-200",
                            "border-gray-100 hover:border-purple-500 hover:bg-purple-50 dark:border-gray-800 dark:hover:border-purple-500 dark:hover:bg-purple-900/10",
                            "group"
                        )}
                    >
                        <div className="w-10 h-10 bg-purple-100 dark:bg-purple-900/30 rounded-full flex items-center justify-center mr-4">
                            <span className="text-xl">🚀</span>
                        </div>
                        <div className="flex-1 text-left">
                            <h3 className="font-semibold text-gray-900 dark:text-white group-hover:text-purple-600 dark:group-hover:text-purple-400">
                                Freighter
                            </h3>
                            <p className="text-sm text-gray-500">
                                {hasFreighter ? 'Connect with browser extension' : 'Install Freighter Extension'}
                            </p>
                        </div>
                    </button>

                    <button
                        disabled
                        className="w-full flex items-center p-4 rounded-xl border-2 border-dashed border-gray-200 dark:border-gray-800 opacity-60 cursor-not-allowed"
                    >
                        <div className="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center mr-4">
                            <span className="text-xl">🔗</span>
                        </div>
                        <div className="flex-1 text-left">
                            <h3 className="font-semibold text-gray-900 dark:text-white">WalletConnect</h3>
                            <p className="text-sm text-gray-500">Coming soon</p>
                        </div>
                    </button>
                </div>

                <div className="p-4 bg-gray-50 dark:bg-gray-800/50 text-center">
                    <p className="text-xs text-gray-500">
                        By connecting a wallet, you agree to our Terms of Service and Privacy Policy.
                    </p>
                </div>
            </div>
        </div>
    );
}

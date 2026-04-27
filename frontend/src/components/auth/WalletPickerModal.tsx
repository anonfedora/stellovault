"use client";

import { useEffect, useState } from "react";
import { X, ExternalLink, CheckCircle2, Circle } from "lucide-react";
import { twMerge } from "tailwind-merge";
import type { WalletType } from "@/hooks/useWalletAuth";

interface WalletOption {
  id: WalletType;
  name: string;
  description: string;
  icon: string;
  available: boolean;
  installed?: boolean;
  installUrl?: string;
  comingSoon?: boolean;
}

interface WalletPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConnect: (type: WalletType) => void;
  isConnecting?: boolean;
  connectingType?: WalletType;
}

export function WalletPickerModal({
  isOpen,
  onClose,
  onConnect,
  isConnecting,
  connectingType,
}: WalletPickerModalProps) {
  const [freighterInstalled, setFreighterInstalled] = useState<boolean | null>(null);

  useEffect(() => {
    if (!isOpen) return;
    // Small delay to let the extension inject into window
    const timer = setTimeout(() => {
      // @ts-expect-error freighter injected by extension
      setFreighterInstalled(typeof window !== 'undefined' && !!window.freighter);
    }, 100);
    return () => clearTimeout(timer);
  }, [isOpen]);

  if (!isOpen) return null;

  const wallets: WalletOption[] = [
    {
      id: 'freighter',
      name: 'Freighter',
      description: freighterInstalled === null
        ? 'Checking...'
        : freighterInstalled
          ? 'Browser extension — ready to connect'
          : 'Extension not detected — click to install',
      icon: '🚀',
      available: true,
      installed: freighterInstalled ?? false,
      installUrl: 'https://www.freighter.app/',
    },
    {
      id: 'walletconnect',
      name: 'WalletConnect',
      description: 'Mobile & desktop wallets',
      icon: '🔗',
      available: false,
      comingSoon: true,
    },
  ];

  const handleWalletClick = (wallet: WalletOption) => {
    if (!wallet.available || wallet.comingSoon) return;
    if (!wallet.installed && wallet.installUrl) {
      window.open(wallet.installUrl, '_blank', 'noopener,noreferrer');
      return;
    }
    onConnect(wallet.id);
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <div className="bg-white dark:bg-gray-900 rounded-2xl shadow-xl w-full max-w-md overflow-hidden border border-gray-200 dark:border-gray-800">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-800">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
            Connect Wallet
          </h2>
          <button
            onClick={onClose}
            aria-label="Close"
            className="p-2 text-gray-400 hover:text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-full transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Wallet options */}
        <div className="p-4 space-y-3">
          {wallets.map((wallet) => {
            const isThisConnecting = isConnecting && connectingType === wallet.id;
            const disabled = !wallet.available || wallet.comingSoon || isConnecting;

            return (
              <button
                key={wallet.id}
                onClick={() => handleWalletClick(wallet)}
                disabled={disabled}
                className={twMerge(
                  "w-full flex items-center p-4 rounded-xl border-2 transition-all duration-200 text-left",
                  wallet.comingSoon
                    ? "border-dashed border-gray-200 dark:border-gray-800 opacity-50 cursor-not-allowed"
                    : !wallet.installed && wallet.installUrl
                      ? "border-amber-200 hover:border-amber-400 hover:bg-amber-50 dark:border-amber-900 dark:hover:border-amber-600 dark:hover:bg-amber-900/10 cursor-pointer"
                      : "border-gray-100 hover:border-purple-500 hover:bg-purple-50 dark:border-gray-800 dark:hover:border-purple-500 dark:hover:bg-purple-900/10 cursor-pointer",
                  isThisConnecting && "opacity-75 cursor-wait",
                  disabled && !wallet.comingSoon && "cursor-not-allowed",
                )}
              >
                <div className={twMerge(
                  "w-10 h-10 rounded-full flex items-center justify-center mr-4 shrink-0",
                  wallet.id === 'freighter' ? "bg-purple-100 dark:bg-purple-900/30" : "bg-blue-100 dark:bg-blue-900/30"
                )}>
                  <span className="text-xl" role="img" aria-hidden>{wallet.icon}</span>
                </div>

                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-gray-900 dark:text-white">
                      {wallet.name}
                    </span>
                    {wallet.comingSoon && (
                      <span className="text-xs px-2 py-0.5 bg-gray-100 dark:bg-gray-800 text-gray-500 rounded-full">
                        Soon
                      </span>
                    )}
                    {!wallet.comingSoon && wallet.installed && (
                      <CheckCircle2 className="w-4 h-4 text-green-500 shrink-0" />
                    )}
                    {!wallet.comingSoon && !wallet.installed && wallet.installUrl && (
                      <ExternalLink className="w-3.5 h-3.5 text-amber-500 shrink-0" />
                    )}
                  </div>
                  <p className="text-sm text-gray-500 truncate">
                    {isThisConnecting ? 'Connecting...' : wallet.description}
                  </p>
                </div>

                {!wallet.comingSoon && (
                  <div className="ml-3 shrink-0">
                    {wallet.installed
                      ? <Circle className="w-4 h-4 text-gray-300 dark:text-gray-600" />
                      : null
                    }
                  </div>
                )}
              </button>
            );
          })}
        </div>

        <div className="px-4 pb-4 text-center">
          <p className="text-xs text-gray-400">
            Non-custodial &amp; secure. We never store your private keys.
          </p>
        </div>
      </div>
    </div>
  );
}

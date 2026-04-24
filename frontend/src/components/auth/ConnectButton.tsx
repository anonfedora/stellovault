"use client";

import { useState } from 'react';
import { useWalletAuth, type WalletType } from '@/hooks/useWalletAuth';
import { WalletPickerModal } from './WalletPickerModal';
import { Loader2, Wallet, AlertCircle, LogOut, ChevronDown } from 'lucide-react';
import { twMerge } from 'tailwind-merge';

interface ConnectButtonProps {
  /** Compact mode for navbar usage */
  compact?: boolean;
}

export function ConnectButton({ compact = false }: ConnectButtonProps) {
  const {
    isConnected,
    isConnecting,
    isCheckingSession,
    publicKey,
    login,
    logout,
    connect,
    error,
    clearError,
  } = useWalletAuth();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [connectingType, setConnectingType] = useState<WalletType>(null);
  const [showDisconnect, setShowDisconnect] = useState(false);

  const handleConnect = async (type: WalletType) => {
    setConnectingType(type);
    setIsModalOpen(false);
    const key = await connect(type);
    if (key) await login(key);
    setConnectingType(null);
  };

  if (isCheckingSession) {
    return (
      <div className="flex items-center gap-2 px-4 py-2 rounded-full bg-gray-100 dark:bg-gray-800 animate-pulse">
        <div className="w-2 h-2 rounded-full bg-gray-300 dark:bg-gray-600" />
        <span className="text-sm text-gray-400 w-20 h-4 bg-gray-200 dark:bg-gray-700 rounded" />
      </div>
    );
  }

  if (isConnected && publicKey) {
    const truncated = `${publicKey.slice(0, 4)}...${publicKey.slice(-4)}`;

    if (compact) {
      return (
        <div className="relative">
          <button
            onClick={() => setShowDisconnect((v) => !v)}
            className="flex items-center gap-2 px-3 py-1.5 bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 rounded-full text-sm font-medium transition-colors"
          >
            <div className="w-2 h-2 rounded-full bg-green-500" />
            <span>{truncated}</span>
            <ChevronDown className="w-3 h-3 text-gray-400" />
          </button>
          {showDisconnect && (
            <div className="absolute right-0 mt-1 w-40 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-xl shadow-lg overflow-hidden z-50">
              <button
                onClick={() => { setShowDisconnect(false); logout(); }}
                className="w-full flex items-center gap-2 px-4 py-2.5 text-sm text-red-500 hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors"
              >
                <LogOut className="w-4 h-4" />
                Disconnect
              </button>
            </div>
          )}
        </div>
      );
    }

    return (
      <div className="flex items-center gap-2">
        <button
          onClick={logout}
          className="flex items-center gap-2 px-4 py-2 bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 rounded-full text-sm font-medium transition-colors"
        >
          <div className="w-2 h-2 rounded-full bg-green-500" />
          <span>{truncated}</span>
          <LogOut className="w-3.5 h-3.5 text-gray-400 ml-1" />
        </button>
      </div>
    );
  }

  return (
    <>
      <button
        onClick={() => { clearError(); setIsModalOpen(true); }}
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

      {error && (
        <div
          role="alert"
          className="mt-3 flex items-start gap-2 text-sm text-red-500 max-w-xs"
        >
          <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
          <span>{error}</span>
        </div>
      )}

      <WalletPickerModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onConnect={handleConnect}
        isConnecting={isConnecting}
        connectingType={connectingType}
      />
    </>
  );
}

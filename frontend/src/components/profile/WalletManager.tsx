'use client';

import { useState } from 'react';
import { Plus, Wallet, Trash2, Star, Copy, Check } from 'lucide-react';
import { Wallet as WalletType } from '@/hooks/useProfile';
import { toast } from 'sonner';

interface WalletManagerProps {
  wallets: WalletType[];
  onAddWallet: (address: string, label?: string) => Promise<void>;
  onRemoveWallet: (walletId: string) => Promise<void>;
  onSetPrimary: (walletId: string) => Promise<void>;
}

export function WalletManager({
  wallets,
  onAddWallet,
  onRemoveWallet,
  onSetPrimary,
}: WalletManagerProps) {
  const [isAdding, setIsAdding] = useState(false);
  const [newWalletAddress, setNewWalletAddress] = useState('');
  const [newWalletLabel, setNewWalletLabel] = useState('');
  const [copiedAddress, setCopiedAddress] = useState<string | null>(null);

  const handleAddWallet = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!newWalletAddress.trim()) {
      toast.error('Please enter a wallet address');
      return;
    }

    try {
      await onAddWallet(newWalletAddress.trim(), newWalletLabel.trim() || undefined);
      setNewWalletAddress('');
      setNewWalletLabel('');
      setIsAdding(false);
      toast.success('Wallet added successfully');
    } catch (error) {
      toast.error('Failed to add wallet');
    }
  };

  const handleRemoveWallet = async (walletId: string) => {
    if (wallets.length === 1) {
      toast.error('You must have at least one wallet');
      return;
    }

    try {
      await onRemoveWallet(walletId);
      toast.success('Wallet removed successfully');
    } catch (error) {
      toast.error('Failed to remove wallet');
    }
  };

  const handleSetPrimary = async (walletId: string) => {
    try {
      await onSetPrimary(walletId);
      toast.success('Primary wallet updated');
    } catch (error) {
      toast.error('Failed to set primary wallet');
    }
  };

  const copyToClipboard = (address: string) => {
    navigator.clipboard.writeText(address);
    setCopiedAddress(address);
    toast.success('Address copied to clipboard');
    setTimeout(() => setCopiedAddress(null), 2000);
  };

  const truncatedAddress = (address: string) => {
    return `${address.slice(0, 8)}...${address.slice(-8)}`;
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center space-x-2">
          <Wallet className="w-5 h-5 text-gray-600" />
          <h2 className="text-lg font-semibold text-gray-900">Wallet Management</h2>
        </div>
        <button
          onClick={() => setIsAdding(!isAdding)}
          className="flex items-center space-x-2 px-4 py-2 text-sm font-medium text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          <span>Add Wallet</span>
        </button>
      </div>

      {isAdding && (
        <form onSubmit={handleAddWallet} className="mb-6 p-4 bg-gray-50 rounded-lg">
          <div className="space-y-4">
            <div>
              <label htmlFor="walletAddress" className="block text-sm font-medium text-gray-700 mb-1">
                Wallet Address
              </label>
              <input
                type="text"
                id="walletAddress"
                value={newWalletAddress}
                onChange={(e) => setNewWalletAddress(e.target.value)}
                placeholder="G..."
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                required
              />
            </div>
            <div>
              <label htmlFor="walletLabel" className="block text-sm font-medium text-gray-700 mb-1">
                Label (Optional)
              </label>
              <input
                type="text"
                id="walletLabel"
                value={newWalletLabel}
                onChange={(e) => setNewWalletLabel(e.target.value)}
                placeholder="e.g., Trading Wallet"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <div className="flex space-x-3">
              <button
                type="submit"
                className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
              >
                Add Wallet
              </button>
              <button
                type="button"
                onClick={() => {
                  setIsAdding(false);
                  setNewWalletAddress('');
                  setNewWalletLabel('');
                }}
                className="px-4 py-2 bg-gray-200 text-gray-700 text-sm font-medium rounded-lg hover:bg-gray-300 transition-colors"
              >
                Cancel
              </button>
            </div>
          </div>
        </form>
      )}

      <div className="space-y-3">
        {wallets.length === 0 ? (
          <p className="text-center text-gray-500 py-8">No wallets connected</p>
        ) : (
          wallets.map((wallet) => (
            <div
              key={wallet.id}
              className="flex items-center justify-between p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
            >
              <div className="flex items-center space-x-4">
                <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-pink-500 rounded-full flex items-center justify-center">
                  <Wallet className="w-5 h-5 text-white" />
                </div>
                <div>
                  <div className="flex items-center space-x-2">
                    <span className="font-medium text-gray-900">
                      {wallet.label || 'Wallet'}
                    </span>
                    {wallet.isPrimary && (
                      <span className="flex items-center space-x-1 px-2 py-0.5 text-xs font-medium bg-yellow-100 text-yellow-800 rounded-full">
                        <Star className="w-3 h-3" />
                        <span>Primary</span>
                      </span>
                    )}
                  </div>
                  <div className="flex items-center space-x-2 mt-1">
                    <code className="text-sm text-gray-600 font-mono">
                      {truncatedAddress(wallet.address)}
                    </code>
                    <button
                      onClick={() => copyToClipboard(wallet.address)}
                      className="text-gray-400 hover:text-gray-600 transition-colors"
                    >
                      {copiedAddress === wallet.address ? (
                        <Check className="w-4 h-4" />
                      ) : (
                        <Copy className="w-4 h-4" />
                      )}
                    </button>
                  </div>
                  {wallet.verifiedAt && (
                    <p className="text-xs text-gray-500 mt-1">
                      Verified {new Date(wallet.verifiedAt).toLocaleDateString()}
                    </p>
                  )}
                </div>
              </div>

              <div className="flex items-center space-x-2">
                {!wallet.isPrimary && (
                  <button
                    onClick={() => handleSetPrimary(wallet.id)}
                    className="p-2 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                    title="Set as primary"
                  >
                    <Star className="w-4 h-4" />
                  </button>
                )}
                <button
                  onClick={() => handleRemoveWallet(wallet.id)}
                  className="p-2 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                  title="Remove wallet"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

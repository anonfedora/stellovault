'use client';

import React, { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Search, AlertCircle } from 'lucide-react';

export default function RiskLookupPage() {
    const [wallet, setWallet] = useState('');
    const [error, setError] = useState('');
    const router = useRouter();

    const handleLookup = (e: React.FormEvent) => {
        e.preventDefault();
        setError('');

        // Basic Stellar address validation
        if (!wallet) {
            setError('Please enter a wallet address');
            return;
        }

        if (!/^G[A-Z2-7]{55}$/.test(wallet)) {
            setError('Invalid Stellar wallet address format');
            return;
        }

        router.push(`/risk/${wallet}`);
    };

    return (
        <div className="min-h-screen bg-white flex flex-col items-center justify-center p-4">
            <div className="max-w-md w-full text-center space-y-8">
                <div className="space-y-4">
                    <div className="mx-auto w-16 h-16 bg-blue-100 flex items-center justify-center rounded-2xl">
                        <Search className="w-8 h-8 text-blue-900" />
                    </div>
                    <h1 className="text-3xl font-bold tracking-tight text-gray-900">
                        Risk Score Lookup
                    </h1>
                    <p className="text-gray-600">
                        Enter a Stellar wallet address to get an instant credit and risk assessment.
                    </p>
                </div>

                <form onSubmit={handleLookup} className="mt-8 space-y-4">
                    <div className="relative">
                        <input
                            type="text"
                            value={wallet}
                            onChange={(e) => setWallet(e.target.value)}
                            placeholder="G..."
                            className={`w-full px-4 py-3 bg-white border ${error ? 'border-red-500 ring-1 ring-red-500' : 'border-gray-200'
                                } rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-900 transition-all text-gray-900 placeholder-gray-400`}
                        />
                        {error && (
                            <div className="absolute -bottom-6 left-0 flex items-center gap-1 text-xs text-red-500">
                                <AlertCircle className="w-3 h-3" />
                                <span>{error}</span>
                            </div>
                        )}
                    </div>

                    <button
                        type="submit"
                        className="w-full bg-blue-900 hover:bg-blue-800 text-white font-semibold py-3 px-6 rounded-xl transition-all shadow-lg shadow-blue-900/10 active:scale-[0.98]"
                    >
                        Assess Risk
                    </button>
                </form>

                <div className="mt-12 grid grid-cols-2 gap-4 text-left">
                    <div className="p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
                        <h3 className="text-sm font-semibold text-gray-900">Protocol Grade</h3>
                        <p className="text-xs text-gray-600 mt-1">Institutional-grade risk assessment based on on-chain reputation.</p>
                    </div>
                    <div className="p-4 bg-white rounded-xl border border-gray-100 shadow-sm">
                        <h3 className="text-sm font-semibold text-gray-900">Real-time Data</h3>
                        <p className="text-xs text-gray-600 mt-1">Live analysis of Stellar ledger activity and asset positions.</p>
                    </div>
                </div>
            </div>
        </div>
    );
}

'use client';

import React, { useState, use } from 'react';
import { useRiskScore } from '../../../hooks/useRiskScore';
import ScoreGauge from '../../../components/risk/ScoreGauge';
import ScoreBreakdown from '../../../components/risk/ScoreBreakdown';
import ScoreHistoryChart from '../../../components/risk/ScoreHistoryChart';
import {
    ArrowLeft,
    TrendingUp,
    ShieldCheck,
    Clock,
    ArrowRight,
    Calculator,
    Info
} from 'lucide-react';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';

export default function WalletRiskPage({ params }: { params: Promise<{ wallet: string }> }) {
    const { wallet: walletAddress } = use(params);
    const searchParams = useSearchParams();
    const startDate = searchParams.get('start_date');
    const endDate = searchParams.get('end_date');

    const { data, loading, error, simulateScore } = useRiskScore(walletAddress);
    const [loanSim, setLoanSim] = useState<string>('0');
    const [projectedScore, setProjectedScore] = useState<number | null>(null);

    const handleSimulate = () => {
        const amount = parseFloat(loanSim);
        if (!isNaN(amount)) {
            setProjectedScore(simulateScore(amount));
        }
    };

    if (loading) {
        return (
            <div className="min-h-screen bg-white flex items-center justify-center">
                <div className="flex flex-col items-center gap-4">
                    <div className="w-12 h-12 border-4 border-blue-900 border-t-transparent rounded-full animate-spin" />
                    <p className="text-gray-500 font-medium">Analyzing Ledger Activity...</p>
                </div>
            </div>
        );
    }

    if (error || !data) {
        return (
            <div className="min-h-screen bg-white flex items-center justify-center p-4">
                <div className="max-w-md w-full bg-white p-8 rounded-2xl shadow-xl border border-red-100 text-center">
                    <div className="w-16 h-16 bg-red-100 text-red-600 rounded-full flex items-center justify-center mx-auto mb-4">
                        <Info className="w-8 h-8" />
                    </div>
                    <h2 className="text-2xl font-bold text-gray-900">{error || 'Wallet Not Found'}</h2>
                    <p className="text-gray-500 mt-2">The Stellar address provided could not be analyzed. Please check the address and try again.</p>
                    <Link
                        href="/risk"
                        className="mt-6 inline-flex items-center gap-2 text-blue-900 font-semibold hover:underline"
                    >
                        <ArrowLeft className="w-4 h-4" /> Back to Search
                    </Link>
                </div>
            </div>
        );
    }

    // Optional: filter history based on dates if provided
    const filteredHistory = data.history.filter(h => {
        if (startDate && h.date < startDate) return false;
        if (endDate && h.date > endDate) return false;
        return true;
    });

    return (
        <div className="min-h-screen bg-gray-50 p-4 md:p-8">
            <div className="max-w-6xl mx-auto space-y-8">
                {/* Navigation & Header */}
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div className="space-y-1">
                        <Link
                            href="/risk"
                            className="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-blue-900 transition-colors mb-2"
                        >
                            <ArrowLeft className="w-4 h-4" /> Back to Assessment
                        </Link>
                        <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-3">
                            Risk Report
                            <span className={`text-[10px] px-2 py-0.5 rounded-full border ${data.grade === 'A' ? 'bg-green-100 text-green-700 border-green-200' :
                                data.grade === 'B' ? 'bg-teal-100 text-teal-700 border-teal-200' :
                                    data.grade === 'C' ? 'bg-yellow-100 text-yellow-700 border-yellow-200' :
                                        'bg-red-100 text-red-700 border-red-200'
                                }`}>
                                MODERN ASSESSMENT
                            </span>
                        </h1>
                        <p className="font-mono text-sm text-gray-600 bg-white border border-gray-100 px-3 py-1 rounded-lg inline-block break-all">
                            {walletAddress}
                        </p>
                    </div>
                    <div className="flex gap-2">
                        <button className="px-4 py-2 bg-white border border-gray-200 rounded-lg text-sm font-medium hover:bg-gray-50 transition-colors">
                            Export PDF
                        </button>
                        <button className="px-4 py-2 bg-blue-900 text-white rounded-lg text-sm font-medium hover:bg-blue-800 transition-colors">
                            Refresh Data
                        </button>
                    </div>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    {/* Main Score & Breakdown */}
                    <div className="lg:col-span-2 space-y-8">
                        <div className="bg-white rounded-2xl p-8 border border-gray-100 shadow-sm flex flex-col md:flex-row items-center gap-12">
                            <ScoreGauge score={data.score} grade={data.grade} size={240} />
                            <div className="flex-1 space-y-4 w-full">
                                <div className="p-4 bg-blue-50/50 rounded-xl border border-blue-100">
                                    <div className="flex items-center gap-2 mb-2 text-blue-900 font-semibold">
                                        <ShieldCheck className="w-5 h-5" />
                                        <span>Protocol Confidence</span>
                                    </div>
                                    <p className="text-gray-600 text-sm leading-relaxed">
                                        This wallet demonstrates a <span className="text-gray-900 font-medium">"{data.grade}" grade </span>
                                        risk profile based on our proprietary on-chain analysis. This assessment is derived from over 48 parameters across the Stellar network.
                                    </p>
                                </div>
                                <div className="flex items-center gap-6 mt-4">
                                    <div className="flex items-center gap-2 text-xs text-gray-500">
                                        <Clock className="w-4 h-4" />
                                        <span>Last updated 2 mins ago</span>
                                    </div>
                                    <div className="flex items-center gap-2 text-xs text-green-600">
                                        <TrendingUp className="w-4 h-4" />
                                        <span>+5 pts since last month</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div className="bg-white rounded-2xl p-8 border border-gray-100 shadow-sm">
                            <ScoreBreakdown breakdown={data.breakdown} />
                        </div>

                        <div className="bg-white rounded-2xl p-8 border border-gray-100 shadow-sm">
                            <div className="flex items-center justify-between mb-6">
                                <h3 className="text-lg font-bold text-gray-900">Historical Trend</h3>
                                <div className="flex gap-2">
                                    <span className="px-2 py-1 bg-gray-100 rounded text-[10px] font-bold text-gray-400">6M</span>
                                    <span className="px-2 py-1 rounded text-[10px] font-bold text-gray-400 hover:bg-gray-100 cursor-pointer">1Y</span>
                                    <span className="px-2 py-1 rounded text-[10px] font-bold text-gray-400 hover:bg-gray-100 cursor-pointer">ALL</span>
                                </div>
                            </div>
                            <ScoreHistoryChart history={filteredHistory} />
                        </div>
                    </div>

                    {/* Sidebar Tools */}
                    <div className="space-y-8">
                        {/* Simulation Panel */}
                        <div className="bg-white rounded-2xl p-6 border border-gray-100 shadow-sm">
                            <div className="flex items-center gap-2 mb-6">
                                <div className="w-8 h-8 bg-purple-100 text-purple-600 rounded-lg flex items-center justify-center">
                                    <Calculator className="w-5 h-5" />
                                </div>
                                <h3 className="text-lg font-bold text-gray-900">Loan Simulator</h3>
                            </div>

                            <div className="space-y-4">
                                <label className="block text-sm font-medium text-gray-700">
                                    Hypothetical Loan Amount (USDC)
                                </label>
                                <div className="relative">
                                    <input
                                        type="number"
                                        value={loanSim}
                                        onChange={(e) => setLoanSim(e.target.value)}
                                        className="w-full pl-4 pr-12 py-3 bg-gray-50 border border-gray-200 rounded-xl focus:ring-2 focus:ring-purple-600 focus:outline-none text-gray-900"
                                        placeholder="0"
                                    />
                                    <span className="absolute right-4 top-3.5 text-xs font-bold text-gray-400">USDC</span>
                                </div>
                                <button
                                    onClick={handleSimulate}
                                    className="w-full bg-purple-600 hover:bg-purple-700 text-white font-bold py-3 px-4 rounded-xl transition-all shadow-lg shadow-purple-600/10"
                                >
                                    Project Score Delta
                                </button>
                            </div>

                            {projectedScore !== null && (
                                <div className="mt-6 p-4 bg-purple-50 rounded-xl border border-purple-100">
                                    <div className="flex justify-between items-end mb-1">
                                        <span className="text-xs text-purple-600 font-semibold uppercase tracking-wider">Projected Score</span>
                                        <span className="text-2xl font-black text-purple-600">{projectedScore.toFixed(0)}</span>
                                    </div>
                                    <div className="flex items-center gap-1 text-xs text-gray-500">
                                        <span className="font-bold text-red-600">-{(data.score - projectedScore).toFixed(0)} points</span>
                                        <span>estimated impact</span>
                                    </div>
                                    <div className="mt-3 w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
                                        <div
                                            className="h-full bg-purple-500"
                                            style={{ width: `${(projectedScore / 1000) * 100}%` }}
                                        />
                                    </div>
                                </div>
                            )}
                        </div>

                        {/* Recommendations */}
                        <div className="bg-white rounded-2xl p-6 border border-gray-100 shadow-sm">
                            <h3 className="text-lg font-bold text-gray-900 mb-4">Improve Score</h3>
                            <div className="space-y-4">
                                <div className="flex gap-4 p-3 hover:bg-gray-50 rounded-xl transition-colors cursor-pointer group">
                                    <div className="w-10 h-10 bg-green-100 text-green-600 rounded-full flex items-center justify-center shrink-0">
                                        <ShieldCheck className="w-5 h-5" />
                                    </div>
                                    <div>
                                        <h4 className="text-sm font-bold text-gray-900">Verify Identity</h4>
                                        <p className="text-xs text-gray-600 mt-0.5">Linking a Stellar SEP-10 domain can boost score by 40+ points.</p>
                                    </div>
                                    <ArrowRight className="w-4 h-4 text-gray-300 group-hover:text-blue-900 self-center ml-auto" />
                                </div>
                                <div className="flex gap-4 p-3 hover:bg-gray-50 rounded-xl transition-colors cursor-pointer group">
                                    <div className="w-10 h-10 bg-blue-100 text-blue-900 rounded-full flex items-center justify-center shrink-0">
                                        <TrendingUp className="w-5 h-5" />
                                    </div>
                                    <div>
                                        <h4 className="text-sm font-bold text-gray-900">Maintain Liquidity</h4>
                                        <p className="text-xs text-gray-600 mt-0.5">Keeping diverse assets over time reduces volatility risk.</p>
                                    </div>
                                    <ArrowRight className="w-4 h-4 text-gray-300 group-hover:text-blue-900 self-center ml-auto" />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

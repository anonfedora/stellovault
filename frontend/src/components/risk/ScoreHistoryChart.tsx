'use client';

import React from 'react';
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
} from 'recharts';
import { RiskHistoryEntry } from '../../hooks/useRiskScore';

interface ScoreHistoryChartProps {
    history: RiskHistoryEntry[];
    loading?: boolean;
    simulationPoint?: { date: string; score: number } | null;
}

const ScoreHistoryChart: React.FC<ScoreHistoryChartProps> = ({ history, loading, simulationPoint }) => {
    // Build simulation line data: last real point → simulation point
    const simulationData: { date: string; score: number }[] = [];
    if (simulationPoint && history.length > 0) {
        const lastReal = history[history.length - 1];
        simulationData.push({ date: lastReal.date, score: lastReal.score });
        simulationData.push({ date: simulationPoint.date, score: simulationPoint.score });
    }

    const tickFormatter = (str: string) => {
        const date = new Date(str);
        return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    };

    if (loading && history.length === 0) {
        return (
            <div className="h-[300px] w-full mt-4 animate-pulse">
                <div className="h-full w-full bg-gray-100 rounded-xl" />
            </div>
        );
    }

    if (!loading && history.length === 0) {
        return (
            <div className="h-[300px] w-full mt-4 flex items-center justify-center text-gray-400 text-sm">
                No history available
            </div>
        );
    }

    return (
        <div className="relative h-[300px] w-full mt-4">
            {loading && (
                <div className="absolute inset-0 bg-white/60 z-10 flex items-center justify-center rounded-xl">
                    <div className="w-6 h-6 border-2 border-blue-900 border-t-transparent rounded-full animate-spin" />
                </div>
            )}
            <ResponsiveContainer width="100%" height="100%">
                <LineChart margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                    <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#e5e7eb" />
                    <XAxis
                        dataKey="date"
                        type="category"
                        allowDuplicatedCategory={false}
                        axisLine={false}
                        tickLine={false}
                        tick={{ fontSize: 12, fill: '#6b7280' }}
                        dy={10}
                        tickFormatter={tickFormatter}
                    />
                    <YAxis
                        domain={[0, 1000]}
                        axisLine={false}
                        tickLine={false}
                        tick={{ fontSize: 12, fill: '#6b7280' }}
                    />
                    <Tooltip
                        contentStyle={{
                            backgroundColor: '#ffffff',
                            border: '1px solid #e5e7eb',
                            borderRadius: '8px',
                            boxShadow: '0 4px 6px -1px rgb(0 0 0 / 0.1)',
                        }}
                        itemStyle={{ color: '#1e3a8a', fontWeight: 'bold' }}
                        labelStyle={{ color: '#6b7280', marginBottom: '4px' }}
                        labelFormatter={(label) => tickFormatter(String(label))}
                    />
                    {/* Real history line */}
                    <Line
                        data={history}
                        type="monotone"
                        dataKey="score"
                        stroke="#1e3a8a"
                        strokeWidth={3}
                        dot={{ fill: '#1e3a8a', strokeWidth: 2, r: 4, stroke: '#fff' }}
                        activeDot={{ r: 6, strokeWidth: 0 }}
                        animationDuration={1500}
                        name="Score"
                    />
                    {/* Simulation projection line */}
                    {simulationData.length === 2 && (
                        <Line
                            data={simulationData}
                            type="monotone"
                            dataKey="score"
                            stroke="#7c3aed"
                            strokeWidth={2}
                            strokeDasharray="6 3"
                            dot={{ fill: '#7c3aed', strokeWidth: 2, r: 4, stroke: '#fff' }}
                            activeDot={{ r: 6, strokeWidth: 0 }}
                            animationDuration={800}
                            name="Projected"
                        />
                    )}
                </LineChart>
            </ResponsiveContainer>
        </div>
    );
};

export default ScoreHistoryChart;

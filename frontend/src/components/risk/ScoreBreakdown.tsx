import React from 'react';
import { RiskScoreBreakdown } from '../../hooks/useRiskScore';

interface ScoreBreakdownProps {
    breakdown: RiskScoreBreakdown[];
}

const ScoreBreakdown: React.FC<ScoreBreakdownProps> = ({ breakdown }) => {
    return (
        <div className="space-y-6">
            <h3 className="text-lg font-bold text-gray-900">Component Breakdown</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {breakdown.map((item, index) => (
                    <div key={index} className="space-y-2">
                        <div className="flex justify-between items-center">
                            <span className="text-sm font-medium text-gray-600">{item.label}</span>
                            <span className="text-sm font-bold text-gray-900">{item.value}%</span>
                        </div>
                        <div className="h-2 w-full bg-gray-100 rounded-full overflow-hidden">
                            <div
                                className="h-full bg-blue-900 rounded-full transition-all duration-1000 ease-out"
                                style={{ width: `${item.value}%` }}
                            />
                        </div>
                        <p className="text-[10px] text-gray-400 uppercase tracking-wider">Weight: {item.weight * 100}%</p>
                    </div>
                ))}
            </div>
        </div>
    );
};

export default ScoreBreakdown;

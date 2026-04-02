import React from 'react';
import { Info } from 'lucide-react';
import { RiskScoreBreakdown } from '../../hooks/useRiskScore';
import Tooltip from './Tooltip';
import { TOOLTIP_CONFIG } from './tooltipConfig';

interface ScoreBreakdownProps {
    breakdown: RiskScoreBreakdown[];
}

const ScoreBreakdown: React.FC<ScoreBreakdownProps> = ({ breakdown }) => {
    return (
        <div className="space-y-6">
            <h3 className="text-lg font-bold text-gray-900">Component Breakdown</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {breakdown.map((item, index) => {
                    const description = TOOLTIP_CONFIG[item.componentKey];
                    return (
                        <div key={index} className="space-y-2">
                            <div className="flex justify-between items-center">
                                <div className="flex items-center gap-1">
                                    <span className="text-sm font-medium text-gray-600">{item.label}</span>
                                    {description && (
                                        <Tooltip
                                            content={{
                                                componentName: item.label,
                                                weight: item.weight,
                                                description,
                                                score: item.value,
                                            }}
                                        >
                                            <Info
                                                size={14}
                                                className="text-gray-400 hover:text-gray-600 transition-colors"
                                                aria-label={`Info about ${item.label}`}
                                            />
                                        </Tooltip>
                                    )}
                                </div>
                                <span className="text-sm font-bold text-gray-900">{item.value}</span>
                            </div>
                            <div className="h-2 w-full bg-gray-100 rounded-full overflow-hidden">
                                <div
                                    className="h-full bg-blue-900 rounded-full transition-all duration-1000 ease-out"
                                    style={{ width: `${(item.value / 1000) * 100}%` }}
                                />
                            </div>
                            <p className="text-[10px] text-gray-400 uppercase tracking-wider">Weight: {item.weight * 100}%</p>
                        </div>
                    );
                })}
            </div>
        </div>
    );
};

export default ScoreBreakdown;

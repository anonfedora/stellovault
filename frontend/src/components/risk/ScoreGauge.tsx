import React from 'react';

interface ScoreGaugeProps {
    score: number;
    grade: 'A' | 'B' | 'C' | 'D' | 'F';
    size?: number;
}

const ScoreGauge: React.FC<ScoreGaugeProps> = ({ score, grade, size = 200 }) => {
    const radius = size * 0.4;
    const stroke = size * 0.08;
    const normalizedRadius = radius - stroke * 2;
    const circumference = normalizedRadius * 2 * Math.PI;
    const strokeDashoffset = circumference - (score / 1000) * circumference;

    const colorMap = {
        A: '#10b981', // green-500
        B: '#14b8a6', // teal-500
        C: '#f59e0b', // amber-500
        D: '#f97316', // orange-500
        F: '#ef4444', // red-500
    };

    const color = colorMap[grade];

    return (
        <div className="relative flex items-center justify-center" style={{ width: size, height: size }}>
            <svg
                height={size}
                width={size}
                className="transform -rotate-90"
            >
                {/* Background circle */}
                <circle
                    stroke="#e5e7eb"
                    fill="transparent"
                    strokeWidth={stroke}
                    r={normalizedRadius}
                    cx={size / 2}
                    cy={size / 2}
                />
                {/* Progress circle */}
                <circle
                    stroke={color}
                    fill="transparent"
                    strokeWidth={stroke}
                    strokeDasharray={circumference + ' ' + circumference}
                    style={{ strokeDashoffset, transition: 'stroke-dashoffset 0.5s ease-in-out' }}
                    strokeLinecap="round"
                    r={normalizedRadius}
                    cx={size / 2}
                    cy={size / 2}
                />
            </svg>
            <div className="absolute inset-0 flex flex-col items-center justify-center mt-2">
                <span className="text-4xl font-bold" style={{ color }}>{score}</span>
                <span className="text-xl font-medium text-gray-500">Grade {grade}</span>
            </div>
        </div>
    );
};

export default ScoreGauge;

import { useState, useEffect } from 'react';

export interface RiskScoreBreakdown {
  label: string;
  value: number; // 0-100
  weight: number;
}

export interface RiskHistoryEntry {
  date: string;
  score: number;
}

export interface RiskScoreData {
  score: number; // 0-1000
  grade: 'A' | 'B' | 'C' | 'D' | 'F';
  breakdown: RiskScoreBreakdown[];
  history: RiskHistoryEntry[];
}

export const useRiskScore = (walletAddress: string | null) => {
  const [data, setData] = useState<RiskScoreData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!walletAddress) {
      setData(null);
      return;
    }

    const fetchRiskScore = async () => {
      setLoading(true);
      setError(null);
      try {
        // Mocking API call
        await new Promise((resolve) => setTimeout(resolve, 1000));

        // Basic validation for Stellar address (standard G...)
        if (!/^G[A-Z2-7]{55}$/.test(walletAddress)) {
          throw new Error('Invalid Stellar wallet address');
        }

        // Generate mock data based on wallet address for consistency in demo
        const hash = walletAddress.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
        const baseScore = 600 + (hash % 350);
        
        const mockData: RiskScoreData = {
          score: baseScore,
          grade: getGrade(baseScore),
          breakdown: [
            { label: 'On-chain Activity', value: 70 + (hash % 25), weight: 0.3 },
            { label: 'Asset Diversity', value: 60 + (hash % 30), weight: 0.2 },
            { label: 'Wallet Longevity', value: 80 + (hash % 15), weight: 0.25 },
            { label: 'Transaction History', value: 65 + (hash % 30), weight: 0.25 },
          ],
          history: Array.from({ length: 6 }).map((_, i) => ({
            date: new Date(Date.now() - (5 - i) * 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
            score: baseScore - (5 - i) * 10 + (Math.random() * 20 - 10),
          })),
        };

        setData(mockData);
      } catch (err: any) {
        setError(err.message || 'Failed to fetch risk score');
        setData(null);
      } finally {
        setLoading(false);
      }
    };

    fetchRiskScore();
  }, [walletAddress]);

  const simulateScore = (loanAmount: number): number => {
    if (!data) return 0;
    // Simple simulation: more loan = higher risk = lower score
    const impact = Math.min(loanAmount / 1000, 100);
    return Math.max(0, data.score - impact);
  };

  return { data, loading, error, simulateScore };
};

const getGrade = (score: number): 'A' | 'B' | 'C' | 'D' | 'F' => {
  if (score >= 850) return 'A';
  if (score >= 700) return 'B';
  if (score >= 550) return 'C';
  if (score >= 400) return 'D';
  return 'F';
};

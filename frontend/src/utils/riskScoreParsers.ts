/**
 * Utilities for mapping backend API responses to frontend types.
 * Implements safe defaults for unexpected field types (Req 7.1, 7.5).
 */

import type {
  RiskScoreData,
  RiskScoreBreakdown,
  RiskHistoryEntry,
} from '../hooks/useRiskScore';

// Backend shape for the v2 risk score response
export interface RiskScoreResponseV2 {
  overall_score: unknown;
  risk_tier: unknown;
  components: unknown;
  confidence: unknown;
  calculated_at: unknown;
}

// Backend shape for a single historical score entry
export interface HistoricalScore {
  date: unknown;
  score: unknown;
  tier: unknown;
}

// Backend shape for the simulate response
export interface SimulateResponse {
  projected_score: unknown;
  scenario_description: unknown;
}

const TIER_TO_GRADE: Record<string, RiskScoreData['grade']> = {
  excellent: 'A',
  good: 'B',
  fair: 'C',
  poor: 'D',
  very_poor: 'F',
  high_risk: 'F',
  unscored: 'F',
};

function safeNumber(value: unknown, fieldName: string, defaultValue = 0): number {
  if (typeof value === 'number' && isFinite(value)) return value;
  console.warn(`[parseRiskScoreResponse] Unexpected type for field "${fieldName}": ${typeof value}. Using default ${defaultValue}.`);
  return defaultValue;
}

function safeString(value: unknown, fieldName: string, defaultValue = ''): string {
  if (typeof value === 'string') return value;
  console.warn(`[parseRiskScoreResponse] Unexpected type for field "${fieldName}": ${typeof value}. Using default "${defaultValue}".`);
  return defaultValue;
}

/**
 * Maps a RiskScoreResponseV2 from the backend to the frontend RiskScoreData type.
 * Uses safe defaults for any field with an unexpected type (Req 7.1, 7.5).
 */
export function parseRiskScoreResponse(raw: RiskScoreResponseV2): RiskScoreData {
  const score = safeNumber(raw.overall_score, 'overall_score');
  const tierRaw = safeString(raw.risk_tier, 'risk_tier');
  const grade: RiskScoreData['grade'] = TIER_TO_GRADE[tierRaw] ?? 'F';
  const confidence = safeNumber(raw.confidence, 'confidence');
  const calculatedAt = safeString(raw.calculated_at, 'calculated_at');

  let breakdown: RiskScoreBreakdown[] = [];
  if (raw.components !== null && typeof raw.components === 'object' && !Array.isArray(raw.components)) {
    const components = raw.components as Record<string, unknown>;
    breakdown = Object.entries(components).map(([key, componentRaw]) => {
      const component =
        componentRaw !== null && typeof componentRaw === 'object' && !Array.isArray(componentRaw)
          ? (componentRaw as Record<string, unknown>)
          : {};
      const value = safeNumber(component.score, `components.${key}.score`);
      const weight = safeNumber(component.weight, `components.${key}.weight`);
      // Convert snake_case key to a human-readable label
      const label = key.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
      return { label, componentKey: key, value, weight };
    });
  } else if (raw.components !== undefined && raw.components !== null) {
    console.warn('[parseRiskScoreResponse] Unexpected type for field "components". Using default [].');
  }

  return {
    score,
    grade,
    breakdown,
    history: [],
    confidence,
    calculatedAt,
  };
}

/**
 * Maps an array of HistoricalScore objects from the backend to RiskHistoryEntry[].
 * Preserves date and score; drops tier (Req 1.5, 7.2).
 */
export function parseHistoricalScores(raw: HistoricalScore[]): RiskHistoryEntry[] {
  if (!Array.isArray(raw)) {
    console.warn('[parseHistoricalScores] Expected array, got:', typeof raw);
    return [];
  }
  return raw.map((item, index) => {
    const date = safeString(item.date, `history[${index}].date`);
    const score = safeNumber(item.score, `history[${index}].score`);
    return { date, score };
  });
}

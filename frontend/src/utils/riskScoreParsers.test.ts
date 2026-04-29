import assert from 'node:assert/strict'
import {
  parseRiskScoreResponse,
  parseHistoricalScores,
} from './riskScoreParsers'

interface TestCase {
  name: string
  run: () => void | Promise<void>
}

export const tests: TestCase[] = [
  // parseRiskScoreResponse
  {
    name: 'parseRiskScoreResponse: maps valid v2 response correctly',
    run: () => {
      const input = {
        overall_score: 78,
        risk_tier: 'good',
        components: { credit: { score: 80, weight: 0.4 } },
        confidence: 0.92,
        calculated_at: '2024-01-15T10:00:00Z',
      }
      const result = parseRiskScoreResponse(input)
      assert.equal(result.score, 78)
      assert.equal(result.grade, 'B')
      assert.ok(result.calculatedAt.includes('2024'))
      assert.equal(result.breakdown.length, 1)
    },
  },
  {
    name: 'parseRiskScoreResponse: uses safe defaults for non-numeric score',
    run: () => {
      const result = parseRiskScoreResponse({
        overall_score: 'not-a-number',
        risk_tier: 'unknown',
        components: null,
        confidence: null,
        calculated_at: null,
      })
      assert.equal(result.score, 0)
      assert.equal(result.grade, 'F')
      assert.deepEqual(result.breakdown, [])
    },
  },
  {
    name: 'parseRiskScoreResponse: maps tier excellent → grade A',
    run: () => {
      const result = parseRiskScoreResponse({
        overall_score: 95,
        risk_tier: 'excellent',
        components: [],
        confidence: 0.99,
        calculated_at: '2024-06-01T00:00:00Z',
      })
      assert.equal(result.grade, 'A')
    },
  },
  {
    name: 'parseRiskScoreResponse: maps tier poor → grade D',
    run: () => {
      const result = parseRiskScoreResponse({
        overall_score: 40,
        risk_tier: 'poor',
        components: [],
        confidence: 0.5,
        calculated_at: '2024-06-01T00:00:00Z',
      })
      assert.equal(result.grade, 'D')
    },
  },
  {
    name: 'parseRiskScoreResponse: handles components object with partial fields',
    run: () => {
      const result = parseRiskScoreResponse({
        overall_score: 60,
        risk_tier: 'fair',
        components: { liquidity: { score: 55 } }, // weight missing
        confidence: 0.7,
        calculated_at: '2024-01-01T00:00:00Z',
      })
      assert.equal(result.breakdown.length, 1)
      assert.equal(result.breakdown[0].value, 55)
      assert.equal(result.breakdown[0].weight, 0) // safe default
    },
  },

  // parseHistoricalScores
  {
    name: 'parseHistoricalScores: maps array of historical entries',
    run: () => {
      const input = [
        { date: '2024-01', score: 70, tier: 'fair' },
        { date: '2024-02', score: 75, tier: 'good' },
      ]
      const result = parseHistoricalScores(input)
      assert.equal(result.length, 2)
      assert.equal(result[0].score, 70)
      assert.equal(result[1].score, 75)
    },
  },
  {
    name: 'parseHistoricalScores: returns empty array for non-array input',
    run: () => {
      assert.deepEqual(parseHistoricalScores(null), [])
      assert.deepEqual(parseHistoricalScores('bad'), [])
      assert.deepEqual(parseHistoricalScores(undefined), [])
    },
  },
  {
    name: 'parseHistoricalScores: uses safe defaults for bad entry fields',
    run: () => {
      const result = parseHistoricalScores([{ date: null, score: 'oops', tier: 42 }])
      assert.equal(result.length, 1)
      assert.equal(result[0].score, 0)
      assert.equal(typeof result[0].date, 'string')
    },
  },

]

async function run() {
  let passed = 0
  let failed = 0
  for (const t of tests) {
    try {
      await t.run()
      console.log(`PASS ${t.name}`)
      passed++
    } catch (err) {
      console.error(`FAIL ${t.name}`)
      console.error(err)
      failed++
    }
  }
  console.log(`\n${passed} passed, ${failed} failed`)
  if (failed > 0) process.exit(1)
}

if (require.main === module) run()

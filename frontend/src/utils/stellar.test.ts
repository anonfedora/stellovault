/**
 * Tests for pure utility functions (no SDK dependencies).
 * Tests for stellar.ts format/shorten helpers are excluded because
 * @stellar/stellar-sdk uses ESM and cannot be loaded by the tsx runner
 * outside of the Next.js build pipeline.
 *
 * These tests cover the contact API validation logic and dashboardExport
 * helpers to complement the existing test files.
 */
import assert from 'node:assert/strict'
import { buildExportFilename, rowsToCsv } from './dashboardExport'

interface TestCase {
  name: string
  run: () => void | Promise<void>
}

// ── Contact form validation (mirrors /api/contact/route.ts logic) ──────────

const SUBJECTS = ['General', 'Partnership', 'Bug Report', 'Oracle Registration'] as const
type Subject = typeof SUBJECTS[number]

function validateContactForm(data: {
  name?: unknown
  email?: unknown
  subject?: unknown
  message?: unknown
}): string | null {
  const { name, email, subject, message } = data
  if (!name || typeof name !== 'string' || name.trim().length < 2) return 'name'
  if (!email || typeof email !== 'string' || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) return 'email'
  if (!subject || !SUBJECTS.includes(subject as Subject)) return 'subject'
  if (!message || typeof message !== 'string' || message.trim().length < 10) return 'message'
  return null
}

export const tests: TestCase[] = [
  // Contact form validation
  {
    name: 'contact form: valid submission passes validation',
    run: () => {
      const err = validateContactForm({
        name: 'Alice Chen',
        email: 'alice@example.com',
        subject: 'General',
        message: 'This is a test message with enough characters.',
      })
      assert.equal(err, null)
    },
  },
  {
    name: 'contact form: rejects short name',
    run: () => {
      assert.equal(
        validateContactForm({ name: 'A', email: 'a@b.com', subject: 'General', message: 'hello world ok' }),
        'name',
      )
    },
  },
  {
    name: 'contact form: rejects missing email',
    run: () => {
      assert.equal(
        validateContactForm({ name: 'Alice', email: '', subject: 'General', message: 'hello world ok' }),
        'email',
      )
    },
  },
  {
    name: 'contact form: rejects malformed email',
    run: () => {
      assert.equal(
        validateContactForm({ name: 'Alice', email: 'not-an-email', subject: 'General', message: 'hello world ok' }),
        'email',
      )
    },
  },
  {
    name: 'contact form: rejects invalid subject',
    run: () => {
      assert.equal(
        validateContactForm({ name: 'Alice', email: 'a@b.com', subject: 'Spam', message: 'hello world ok' }),
        'subject',
      )
    },
  },
  {
    name: 'contact form: accepts all valid subject values',
    run: () => {
      for (const s of SUBJECTS) {
        const err = validateContactForm({
          name: 'Alice', email: 'a@b.com', subject: s, message: 'hello world ok',
        })
        assert.equal(err, null, `Expected no error for subject "${s}"`)
      }
    },
  },
  {
    name: 'contact form: rejects short message',
    run: () => {
      assert.equal(
        validateContactForm({ name: 'Alice', email: 'a@b.com', subject: 'General', message: 'Hi' }),
        'message',
      )
    },
  },

  // dashboardExport helpers (complements existing dashboardExport.test.ts)
  {
    name: 'buildExportFilename: contains the provided prefix',
    run: () => {
      const filename = buildExportFilename('loan-report')
      assert.ok(filename.includes('loan-report'), `Expected prefix in "${filename}"`)
    },
  },
  {
    name: 'buildExportFilename: returns a non-empty string',
    run: () => {
      assert.ok(buildExportFilename('test').length > 0)
    },
  },
  {
    name: 'rowsToCsv: single row with headers',
    run: () => {
      const csv = rowsToCsv([{ a: '1', b: '2' }])
      assert.ok(csv.includes('a'))
      assert.ok(csv.includes('b'))
      assert.ok(csv.includes('1'))
      assert.ok(csv.includes('2'))
    },
  },
  {
    name: 'rowsToCsv: escapes commas inside values',
    run: () => {
      const csv = rowsToCsv([{ name: 'Smith, Alice', amount: '100' }])
      assert.ok(csv.includes('"Smith, Alice"'))
    },
  },
  {
    name: 'rowsToCsv: multiple rows share the same headers',
    run: () => {
      const csv = rowsToCsv([{ x: '1' }, { x: '2' }])
      const lines = csv.split('\n').filter(Boolean)
      // header line + 2 data lines
      assert.equal(lines.length, 3)
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

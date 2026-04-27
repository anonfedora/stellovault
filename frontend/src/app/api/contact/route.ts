import { NextRequest, NextResponse } from 'next/server'

const SUBJECTS = ['General', 'Partnership', 'Bug Report', 'Oracle Registration'] as const

// Simple in-memory rate limiter: max 5 submissions per IP per 15 min
const rateLimitMap = new Map<string, { count: number; resetAt: number }>()

function checkRateLimit(ip: string): boolean {
  const now = Date.now()
  const windowMs = 15 * 60 * 1000
  const limit = 5
  const entry = rateLimitMap.get(ip)

  if (!entry || entry.resetAt < now) {
    rateLimitMap.set(ip, { count: 1, resetAt: now + windowMs })
    return true
  }
  if (entry.count >= limit) return false
  entry.count++
  return true
}

export async function POST(req: NextRequest) {
  const ip = req.headers.get('x-forwarded-for') ?? req.headers.get('x-real-ip') ?? 'unknown'

  if (!checkRateLimit(ip)) {
    return NextResponse.json(
      { error: 'Too many requests. Please try again later.' },
      { status: 429 },
    )
  }

  let body: Record<string, unknown>
  try {
    body = await req.json()
  } catch {
    return NextResponse.json({ error: 'Invalid JSON body' }, { status: 400 })
  }

  const { name, email, subject, message } = body

  if (!name || typeof name !== 'string' || name.trim().length < 2) {
    return NextResponse.json({ error: 'Name must be at least 2 characters' }, { status: 400 })
  }
  if (!email || typeof email !== 'string' || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    return NextResponse.json({ error: 'Valid email address is required' }, { status: 400 })
  }
  if (!subject || !SUBJECTS.includes(subject as typeof SUBJECTS[number])) {
    return NextResponse.json(
      { error: `Subject must be one of: ${SUBJECTS.join(', ')}` },
      { status: 400 },
    )
  }
  if (!message || typeof message !== 'string' || message.trim().length < 10) {
    return NextResponse.json({ error: 'Message must be at least 10 characters' }, { status: 400 })
  }

  // In production, send to email service / CRM here
  console.info('[contact] submission', { name, email: email.replace(/@.*/, '@***'), subject })

  return NextResponse.json({ ok: true, message: 'Your message has been received.' })
}

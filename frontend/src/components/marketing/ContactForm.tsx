'use client'

import React, { useState } from 'react'
import { Send, CheckCircle2, Loader2 } from 'lucide-react'
import { toast } from 'sonner'

const SUBJECTS = ['General', 'Partnership', 'Bug Report', 'Oracle Registration'] as const

interface FormData {
  name: string
  email: string
  subject: string
  message: string
}

const EMPTY: FormData = { name: '', email: '', subject: 'General', message: '' }

export function ContactForm() {
  const [form, setForm] = useState<FormData>(EMPTY)
  const [status, setStatus] = useState<'idle' | 'loading' | 'success'>('idle')

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
    setForm(prev => ({ ...prev, [e.target.name]: e.target.value }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setStatus('loading')
    try {
      const res = await fetch('/api/contact', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(form),
      })
      const data = await res.json()
      if (!res.ok) throw new Error(data.error ?? 'Submission failed')
      setStatus('success')
      setForm(EMPTY)
      toast.success('Message sent! We\'ll get back to you within 24 hours.')
    } catch (err) {
      setStatus('idle')
      toast.error(err instanceof Error ? err.message : 'Something went wrong. Please try again.')
    }
  }

  if (status === 'success') {
    return (
      <div className="flex flex-col items-center justify-center gap-4 py-16 text-center">
        <CheckCircle2 className="h-14 w-14 text-green-500" />
        <h3 className="text-xl font-bold text-gray-900">Message received!</h3>
        <p className="text-gray-500">We&apos;ll get back to you within 24 hours.</p>
        <button
          onClick={() => setStatus('idle')}
          className="mt-2 text-sm text-blue-900 underline underline-offset-4 hover:text-blue-700"
        >
          Send another message
        </button>
      </div>
    )
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-5">
      <div className="grid sm:grid-cols-2 gap-5">
        <div>
          <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1.5">
            Full name <span className="text-red-500">*</span>
          </label>
          <input
            id="name"
            name="name"
            type="text"
            required
            minLength={2}
            value={form.name}
            onChange={handleChange}
            placeholder="Alice Chen"
            className="w-full px-4 py-3 rounded-xl border border-gray-200 focus:ring-2 focus:ring-blue-900 focus:border-transparent outline-none text-sm transition-all"
          />
        </div>
        <div>
          <label htmlFor="email" className="block text-sm font-medium text-gray-700 mb-1.5">
            Email address <span className="text-red-500">*</span>
          </label>
          <input
            id="email"
            name="email"
            type="email"
            required
            value={form.email}
            onChange={handleChange}
            placeholder="alice@company.com"
            className="w-full px-4 py-3 rounded-xl border border-gray-200 focus:ring-2 focus:ring-blue-900 focus:border-transparent outline-none text-sm transition-all"
          />
        </div>
      </div>

      <div>
        <label htmlFor="subject" className="block text-sm font-medium text-gray-700 mb-1.5">
          Subject <span className="text-red-500">*</span>
        </label>
        <select
          id="subject"
          name="subject"
          required
          value={form.subject}
          onChange={handleChange}
          className="w-full px-4 py-3 rounded-xl border border-gray-200 focus:ring-2 focus:ring-blue-900 focus:border-transparent outline-none text-sm transition-all bg-white"
        >
          {SUBJECTS.map(s => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>

      <div>
        <label htmlFor="message" className="block text-sm font-medium text-gray-700 mb-1.5">
          Message <span className="text-red-500">*</span>
        </label>
        <textarea
          id="message"
          name="message"
          required
          minLength={10}
          rows={6}
          value={form.message}
          onChange={handleChange}
          placeholder="Tell us how we can help…"
          className="w-full px-4 py-3 rounded-xl border border-gray-200 focus:ring-2 focus:ring-blue-900 focus:border-transparent outline-none text-sm transition-all resize-none"
        />
      </div>

      <button
        type="submit"
        disabled={status === 'loading'}
        className="w-full inline-flex items-center justify-center gap-2 px-6 py-3.5 rounded-xl bg-blue-900 text-white font-semibold text-sm hover:bg-blue-800 disabled:opacity-60 disabled:cursor-not-allowed transition-colors"
      >
        {status === 'loading' ? (
          <><Loader2 className="h-4 w-4 animate-spin" />Sending…</>
        ) : (
          <><Send className="h-4 w-4" />Send message</>
        )}
      </button>
    </form>
  )
}

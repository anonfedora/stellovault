'use client'

import React, { useState } from 'react'
import { Plus, Minus } from 'lucide-react'

interface FaqItem {
  question: string
  answer: string
}

interface Props {
  items: FaqItem[]
}

export function FaqAccordion({ items }: Props) {
  const [open, setOpen] = useState<number | null>(null)

  const toggle = (i: number) => setOpen(prev => (prev === i ? null : i))

  return (
    <div className="divide-y divide-gray-200 rounded-2xl border border-gray-200 overflow-hidden">
      {items.map((item, i) => (
        <div key={i} className="bg-white">
          <button
            onClick={() => toggle(i)}
            aria-expanded={open === i}
            className="w-full flex items-center justify-between gap-4 px-6 py-5 text-left hover:bg-gray-50 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-blue-900"
          >
            <span className="font-semibold text-gray-900 text-sm leading-snug">{item.question}</span>
            <span className="shrink-0 text-blue-900">
              {open === i ? <Minus className="h-4 w-4" /> : <Plus className="h-4 w-4" />}
            </span>
          </button>
          {open === i && (
            <div className="px-6 pb-5 text-sm text-gray-600 leading-relaxed">
              {item.answer}
            </div>
          )}
        </div>
      ))}
    </div>
  )
}

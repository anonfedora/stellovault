
'use client'

import React, { useState } from 'react'
import { Plus, Minus } from 'lucide-react'

const faqs = [
  {
    question: "How do I list collateral?",
    answer: "To list collateral, navigate to the Dashboard, click on 'Tokenize Asset', and upload your trade documents (invoices, bills of lading). Our system will verify the data and create a fractional Stellar token representing your asset."
  },
  {
    question: "What fees apply to borrowers?",
    answer: "StelloVault charges a transparency fee of 0.5% on the total loan amount. There are no hidden intermediary fees, as all transactions happen directly between you and the liquidity providers via smart contracts."
  },
  {
    question: "How are escrows secured?",
    answer: "Escrows are managed by Soroban smart contracts on the Stellar network. Funds are released only when pre-defined conditions (like shipping verification oracles) are met, ensuring trustless execution."
  },
  {
    question: "Can I tokenize any type of trade asset?",
    answer: "Currently, we support verified invoices, warehouse receipts, and shipping receivables. We are expanding to include more complex commodities through community governance voting."
  },
  {
    question: "What is the minimum loan amount?",
    answer: "To ensure economic viability for both parties, the minimum financing request is $5,000 USD equivalent in XLM or USDC."
  },
  {
    question: "How does the intelligent risk scoring work?",
    answer: "Our backend analyzes your on-chain transaction history, collateral quality, and oracle-verified trade data to generate a real-time risk score, which determines your interest rates."
  },
  {
    question: "How do I join the Oracle Registration?",
    answer: "If you are a logistics provider or inspection agency, you can apply through the Oracle Registration section in the Contact form. Oracles play a critical role in verifying physical trade events for our smart contracts."
  }
]

export function FAQ() {
  const [openIndex, setOpenIndex] = useState<number | null>(null)

  return (
    <section className="py-24 px-6 bg-white">
      <div className="max-w-3xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-4xl font-bold text-gray-900 mb-4">Frequently Asked Questions</h2>
          <p className="text-gray-600 font-medium">Everything you need to know about StelloVault trade finance.</p>
        </div>

        <div className="space-y-4">
          {faqs.map((faq, index) => (
            <div 
              key={index} 
              className={`border rounded-2xl transition-all duration-300 ${openIndex === index ? 'border-blue-200 bg-blue-50/50' : 'border-gray-100 hover:border-blue-100'}`}
            >
              <button
                onClick={() => setOpenIndex(openIndex === index ? null : index)}
                className="w-full px-6 py-5 flex items-center justify-between text-left"
              >
                <span className={`font-semibold text-lg ${openIndex === index ? 'text-blue-900' : 'text-gray-900'}`}>
                  {faq.question}
                </span>
                <div className={`flex-shrink-0 ml-4 transition-transform duration-300 ${openIndex === index ? 'rotate-180 text-blue-900' : 'text-gray-400'}`}>
                  {openIndex === index ? <Minus className="w-5 h-5" /> : <Plus className="w-5 h-5" />}
                </div>
              </button>
              
              <div 
                className={`overflow-hidden transition-all duration-300 ease-in-out ${openIndex === index ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'}`}
              >
                <div className="px-6 pb-6 text-gray-600 leading-relaxed">
                  {faq.answer}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

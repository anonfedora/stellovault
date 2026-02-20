import React from 'react'
import { Lock, Shield, TrendingUp, ArrowRight } from 'lucide-react'

const steps = [
  {
    step: 1,
    title: 'Tokenize',
    subtitle: 'Real-World Assets → Stellar Tokens',
    description:
      'SMEs upload trade documents — invoices, warehouse receipts, bills of lading. StelloVault verifies, then mints fractional Stellar tokens with embedded metadata for full traceability.',
    icon: Lock,
    color: 'bg-blue-900',
    lightColor: 'bg-blue-100',
    textColor: 'text-blue-900',
  },
  {
    step: 2,
    title: 'Escrow',
    subtitle: 'Soroban Smart Contract Custody',
    description:
      'Multi-signature escrows, managed entirely on-chain by Soroban contracts. Funds are held securely and released only when oracle-verified conditions (shipment delivery, inspection) are met.',
    icon: Shield,
    color: 'bg-indigo-700',
    lightColor: 'bg-indigo-100',
    textColor: 'text-indigo-700',
  },
  {
    step: 3,
    title: 'Finance',
    subtitle: 'Instant Liquidity for Growth',
    description:
      'Investors bid on tokenized collateral through the marketplace. SMEs receive capital within minutes — no intermediaries, no weeks of waiting. Repayment terms are transparent and on-chain.',
    icon: TrendingUp,
    color: 'bg-cyan-700',
    lightColor: 'bg-cyan-100',
    textColor: 'text-cyan-700',
  },
]

export function HowItWorks() {
  return (
    <section id="how-it-works" className="py-24 px-6 bg-white">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-20 space-y-4">
          <span className="inline-block bg-blue-100 text-blue-900 px-4 py-1.5 rounded-full text-sm font-semibold tracking-wide">
            The Process
          </span>
          <h2 className="text-4xl md:text-5xl font-bold text-gray-900">
            How StelloVault Works
          </h2>
          <p className="text-lg text-gray-600 max-w-2xl mx-auto">
            Three steps from real-world asset to on-chain liquidity
          </p>
        </div>

        {/* Desktop: Horizontal Flow */}
        <div className="hidden lg:grid lg:grid-cols-3 gap-0 relative">
          {/* Connection Lines */}
          <div className="absolute top-16 left-[calc(33.33%-1rem)] w-[calc(33.33%+2rem)] h-0.5 bg-gradient-to-r from-blue-300 via-indigo-300 to-cyan-300 z-0" />
          <div className="absolute top-16 left-[calc(66.66%-1rem)] w-[calc(33.33%+2rem)] h-0.5 bg-gradient-to-r from-indigo-300 to-cyan-300 z-0" />

          {steps.map((item, idx) => (
            <div key={idx} className="relative z-10 flex flex-col items-center text-center px-8">
              {/* Step Number + Icon */}
              <div
                className={`w-32 h-32 rounded-3xl ${item.color} flex flex-col items-center justify-center mb-8 shadow-lg`}
              >
                <span className="text-white/60 text-xs font-bold uppercase tracking-widest mb-1">
                  Step {item.step}
                </span>
                <item.icon className="w-10 h-10 text-white" />
              </div>

              <h3 className="text-2xl font-bold text-gray-900 mb-2">{item.title}</h3>
              <p className={`text-sm font-semibold ${item.textColor} mb-4`}>{item.subtitle}</p>
              <p className="text-gray-600 leading-relaxed text-sm">{item.description}</p>
            </div>
          ))}
        </div>

        {/* Mobile/Tablet: Vertical Flow */}
        <div className="lg:hidden space-y-6">
          {steps.map((item, idx) => (
            <div key={idx} className="relative">
              <div className="flex gap-6">
                <div className="flex flex-col items-center">
                  <div
                    className={`w-16 h-16 rounded-2xl ${item.color} flex items-center justify-center shadow-lg flex-shrink-0`}
                  >
                    <item.icon className="w-7 h-7 text-white" />
                  </div>
                  {idx < steps.length - 1 && (
                    <div className="w-0.5 h-full min-h-[4rem] bg-gradient-to-b from-blue-200 to-transparent mt-3" />
                  )}
                </div>
                <div className="pb-8 pt-1 flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <span
                      className={`${item.lightColor} ${item.textColor} text-xs font-bold px-2.5 py-1 rounded-full`}
                    >
                      Step {item.step}
                    </span>
                  </div>
                  <h3 className="text-xl font-bold text-gray-900 mb-1">{item.title}</h3>
                  <p className={`text-sm font-semibold ${item.textColor} mb-3`}>{item.subtitle}</p>
                  <p className="text-gray-600 leading-relaxed text-sm">{item.description}</p>
                </div>
              </div>

              {idx < steps.length - 1 && (
                <div className="absolute left-8 -translate-x-1/2 bottom-2">
                  <ArrowRight className="w-4 h-4 text-gray-300 rotate-90" />
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

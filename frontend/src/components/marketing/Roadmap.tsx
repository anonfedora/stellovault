import React from 'react'
import { CheckCircle2, Circle, Clock } from 'lucide-react'

type MilestoneStatus = 'completed' | 'in-progress' | 'upcoming'

interface Milestone {
  date: string
  title: string
  description: string
  status: MilestoneStatus
}

const milestones: Milestone[] = [
  {
    date: 'Q3 2025',
    title: 'Protocol Design & Research',
    description:
      'Completed architecture design for collateral tokenization, Soroban smart contract escrow model, and oracle integration specifications.',
    status: 'completed',
  },
  {
    date: 'Q4 2025',
    title: 'Soroban MVP on Testnet',
    description:
      'Deployed initial Soroban escrow contracts on Stellar Testnet. Built frontend prototype with wallet connect and collateral listing flows.',
    status: 'completed',
  },
  {
    date: 'Q1 2026',
    title: 'Oracle & Risk Engine Integration',
    description:
      'Integrated shipment verification oracles and deployed the on-chain risk scoring engine. Launched beta with select SME partners in East Africa.',
    status: 'in-progress',
  },
  {
    date: 'Q2 2026',
    title: 'Mainnet Launch',
    description:
      'Full Stellar Mainnet deployment with audited smart contracts, KYC integration, and the first public liquidity pools for trade financing.',
    status: 'upcoming',
  },
  {
    date: 'Q3 2026',
    title: 'AfCFTA Corridor Expansion',
    description:
      'Expand to key intra-African trade corridors (Nigeria ↔ Kenya, Ghana ↔ South Africa). Onboard institutional liquidity providers.',
    status: 'upcoming',
  },
  {
    date: 'Q4 2026',
    title: 'Governance & DAO Launch',
    description:
      'Launch quadratic voting governance for collateral acceptance. Introduce community-driven protocol parameters and treasury management.',
    status: 'upcoming',
  },
]

const StatusIcon = ({ status }: { status: MilestoneStatus }) => {
  if (status === 'completed')
    return <CheckCircle2 className="w-5 h-5 text-green-500" />
  if (status === 'in-progress')
    return <Clock className="w-5 h-5 text-blue-500 animate-pulse" />
  return <Circle className="w-5 h-5 text-gray-300" />
}

const statusLabel = (status: MilestoneStatus) => {
  if (status === 'completed') return 'Completed'
  if (status === 'in-progress') return 'In Progress'
  return 'Upcoming'
}

const statusBadgeColor = (status: MilestoneStatus) => {
  if (status === 'completed') return 'bg-green-100 text-green-700'
  if (status === 'in-progress') return 'bg-blue-100 text-blue-700'
  return 'bg-gray-100 text-gray-500'
}

export function Roadmap() {
  return (
    <section id="roadmap" className="py-24 px-6 bg-gray-50">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-20 space-y-4">
          <span className="inline-block bg-blue-100 text-blue-900 px-4 py-1.5 rounded-full text-sm font-semibold tracking-wide">
            Roadmap
          </span>
          <h2 className="text-4xl md:text-5xl font-bold text-gray-900">
            Building in Public
          </h2>
          <p className="text-lg text-gray-600 max-w-2xl mx-auto">
            Our journey from protocol design to a fully decentralized trade finance ecosystem
          </p>
        </div>

        <div className="relative">
          {/* Vertical timeline line */}
          <div className="absolute left-6 md:left-1/2 md:-translate-x-px top-0 bottom-0 w-0.5 bg-gradient-to-b from-green-300 via-blue-300 to-gray-200" />

          <div className="space-y-12">
            {milestones.map((milestone, idx) => {
              const isEven = idx % 2 === 0
              return (
                <div
                  key={idx}
                  className={`relative flex items-start gap-6 md:gap-0 ${
                    isEven ? 'md:flex-row' : 'md:flex-row-reverse'
                  }`}
                >
                  {/* Timeline Dot */}
                  <div className="absolute left-6 md:left-1/2 -translate-x-1/2 w-12 h-12 rounded-full bg-white border-2 border-gray-200 flex items-center justify-center z-10 shadow-sm">
                    <StatusIcon status={milestone.status} />
                  </div>

                  {/* Content Card */}
                  <div
                    className={`ml-16 md:ml-0 md:w-[calc(50%-3rem)] ${
                      isEven ? 'md:pr-0 md:text-right' : 'md:pl-0 md:text-left'
                    }`}
                  >
                    <div
                      className={`bg-white rounded-2xl p-6 border border-gray-100 shadow-sm hover:shadow-md hover:border-blue-100 transition-all ${
                        milestone.status === 'in-progress'
                          ? 'ring-2 ring-blue-100'
                          : ''
                      }`}
                    >
                      <div
                        className={`flex items-center gap-3 mb-3 ${
                          isEven ? 'md:justify-end' : 'md:justify-start'
                        }`}
                      >
                        <span className="text-sm font-bold text-gray-900">
                          {milestone.date}
                        </span>
                        <span
                          className={`text-xs font-semibold px-2.5 py-0.5 rounded-full ${statusBadgeColor(
                            milestone.status
                          )}`}
                        >
                          {statusLabel(milestone.status)}
                        </span>
                      </div>
                      <h3 className="text-lg font-bold text-gray-900 mb-2">
                        {milestone.title}
                      </h3>
                      <p className="text-sm text-gray-600 leading-relaxed">
                        {milestone.description}
                      </p>
                    </div>
                  </div>

                  {/* Spacer for the other side */}
                  <div className="hidden md:block md:w-[calc(50%-3rem)]" />
                </div>
              )
            })}
          </div>
        </div>
      </div>
    </section>
  )
}

import React from 'react'
import { Metadata } from 'next'
import { Navbar } from '@/components/layout/Navbar'
import { Footer } from '@/components/layout/Footer'
import { HowItWorks } from '@/components/marketing/HowItWorks'
import { TeamCard, TeamMember } from '@/components/marketing/TeamCard'
import { Roadmap } from '@/components/marketing/Roadmap'
import { CheckCircle2, Zap, Globe, Database, Shield, Cpu } from 'lucide-react'

export const metadata: Metadata = {
  title: 'About StelloVault | Unlocking Trade Finance on Stellar',
  description:
    'Learn about StelloVault — the decentralized trade finance protocol on Stellar. Discover our mission, team, technology, and roadmap for closing the $100B+ trade finance gap.',
}

const teamMembers: TeamMember[] = [
  {
    name: 'Amina Diallo',
    role: 'Co-Founder & CEO',
    bio: 'Former trade finance analyst at AfDB. Passionate about leveraging blockchain to unlock SME growth across Africa.',
    avatar: 'AD',
    github: 'https://github.com',
    twitter: 'https://twitter.com',
  },
  {
    name: 'Kofi Mensah',
    role: 'CTO & Lead Engineer',
    bio: 'Full-stack blockchain engineer with deep expertise in Soroban, Rust, and Stellar protocol internals.',
    avatar: 'KM',
    github: 'https://github.com',
    twitter: 'https://twitter.com',
  },
  {
    name: 'Fatima Ouedraogo',
    role: 'Head of Product',
    bio: 'Product lead specializing in financial inclusion tools. Previously built microfinance platforms for West African markets.',
    avatar: 'FO',
    github: 'https://github.com',
  },
  {
    name: 'David Osei',
    role: 'Smart Contract Lead',
    bio: 'Soroban and Rust specialist focused on escrow contract security, multi-sig logic, and oracle integrations.',
    avatar: 'DO',
    github: 'https://github.com',
    twitter: 'https://twitter.com',
  },
  {
    name: 'Ngozi Eze',
    role: 'Risk & Compliance',
    bio: 'Regulatory compliance expert with experience in DeFi risk modeling and on-chain credit scoring systems.',
    avatar: 'NE',
    twitter: 'https://twitter.com',
  },
  {
    name: 'Tendai Moyo',
    role: 'Community & Partnerships',
    bio: 'Building the StelloVault ecosystem — connecting SMEs, investors, oracle providers, and logistics partners across Africa.',
    avatar: 'TM',
    github: 'https://github.com',
    twitter: 'https://twitter.com',
  },
]

export default function AboutPage() {
  return (
    <main className="min-h-screen bg-white">
      <Navbar />

      {/* ─── Hero ─── */}
      <section className="pt-32 pb-20 px-6 bg-blue-900 text-white relative overflow-hidden">
        {/* Background Glow */}
        <div className="absolute inset-0 overflow-hidden pointer-events-none opacity-20">
          <div className="absolute -top-[15%] -left-[10%] w-[45%] h-[70%] bg-blue-400 blur-[140px] rounded-full" />
          <div className="absolute top-[30%] -right-[10%] w-[50%] h-[60%] bg-cyan-400 blur-[150px] rounded-full" />
        </div>

        <div className="max-w-4xl mx-auto text-center relative z-10 space-y-6">
          <span className="inline-block bg-white/10 backdrop-blur text-blue-100 px-4 py-1.5 rounded-full text-sm font-semibold">
            About StelloVault
          </span>
          <h1 className="text-4xl md:text-6xl font-bold leading-tight">
            Closing the Trade Finance Gap with <span className="text-cyan-300">On‑Chain Innovation</span>
          </h1>
          <p className="text-xl text-blue-100/80 max-w-2xl mx-auto leading-relaxed">
            We&apos;re building the decentralized infrastructure that connects real-world commerce to instant blockchain-powered liquidity on the Stellar network.
          </p>
        </div>
      </section>

      {/* ─── Problem Statement ─── */}
      <section className="py-24 px-6 bg-white">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16 space-y-4">
            <span className="inline-block bg-red-50 text-red-600 px-4 py-1.5 rounded-full text-sm font-semibold">
              The Problem
            </span>
            <h2 className="text-4xl md:text-5xl font-bold text-gray-900">
              A $100–120B Crisis
            </h2>
            <p className="text-lg text-gray-600 max-w-2xl mx-auto">
              The global trade finance gap disproportionately impacts small and medium enterprises in emerging markets.
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                stat: '$100–120B',
                label: 'Annual Financing Gap',
                desc: 'The unmet demand for trade finance globally, concentrated in Africa and Southeast Asia where SMEs drive 90% of business activity.',
                gradient: 'from-red-500 to-orange-500',
              },
              {
                stat: '40%',
                label: 'SME Rejection Rate',
                desc: 'Nearly half of trade finance applications from small businesses are rejected by traditional banks due to lack of collateral history and documentation.',
                gradient: 'from-amber-500 to-yellow-400',
              },
              {
                stat: '$2.5T',
                label: 'AfCFTA Opportunity',
                desc: 'The African Continental Free Trade Area represents a massive opportunity — but realizing it requires accessible financing for cross-border SME trade.',
                gradient: 'from-green-500 to-emerald-400',
              },
            ].map((item, idx) => (
              <div
                key={idx}
                className="relative overflow-hidden rounded-2xl border border-gray-100 p-8 hover:shadow-lg transition-shadow group"
              >
                <div
                  className={`text-4xl md:text-5xl font-black bg-gradient-to-r ${item.gradient} bg-clip-text text-transparent mb-2`}
                >
                  {item.stat}
                </div>
                <h3 className="text-lg font-bold text-gray-900 mb-3">{item.label}</h3>
                <p className="text-sm text-gray-600 leading-relaxed">{item.desc}</p>
                {/* Accent line */}
                <div
                  className={`absolute bottom-0 left-0 h-1 w-0 group-hover:w-full bg-gradient-to-r ${item.gradient} transition-all duration-500`}
                />
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── How It Works ─── */}
      <HowItWorks />

      {/* ─── Technology Stack ─── */}
      <section className="py-24 px-6 bg-gray-50">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16 space-y-4">
            <span className="inline-block bg-blue-100 text-blue-900 px-4 py-1.5 rounded-full text-sm font-semibold tracking-wide">
              Technology
            </span>
            <h2 className="text-4xl md:text-5xl font-bold text-gray-900">
              Built on Best-in-Class Infrastructure
            </h2>
            <p className="text-lg text-gray-600 max-w-2xl mx-auto">
              Every layer of StelloVault is purpose-built for secure, fast, and transparent trade finance.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {[
              {
                icon: Globe,
                title: 'Stellar Protocol',
                desc: 'Sub-second settlement, native multi-asset support, and cross-border payment rails designed for real-world commerce.',
                tag: 'Network',
              },
              {
                icon: Shield,
                title: 'Soroban Smart Contracts',
                desc: 'Rust-based, auditable contracts for escrow custody, conditional releases, and multi-signature verification logic.',
                tag: 'Contracts',
              },
              {
                icon: Cpu,
                title: 'Oracle Integration',
                desc: 'Third-party oracles verify real-world events — shipment tracking, customs clearance, warehouse inspection — before funds are released.',
                tag: 'Data',
              },
              {
                icon: Database,
                title: 'Account Abstraction',
                desc: 'Simplified wallet interactions so SMEs don\'t need deep crypto knowledge. Gas-free meta-transactions for a seamless experience.',
                tag: 'UX',
              },
              {
                icon: Zap,
                title: 'On-Chain Risk Engine',
                desc: 'Algorithmic creditworthiness scoring using transaction history, collateral utilization, and oracle data — fully transparent and auditable.',
                tag: 'Intelligence',
              },
              {
                icon: CheckCircle2,
                title: 'Compliance Layer',
                desc: 'Built-in KYC/AML hooks, regulatory reporting, and audit trails to meet the requirements of institutional liquidity providers.',
                tag: 'Compliance',
              },
            ].map((item, idx) => (
              <div
                key={idx}
                className="bg-white rounded-2xl border border-gray-100 p-8 hover:border-blue-200 hover:shadow-lg transition-all group"
              >
                <div className="flex items-center justify-between mb-6">
                  <div className="w-12 h-12 rounded-xl bg-blue-100 group-hover:bg-blue-900 flex items-center justify-center transition-colors">
                    <item.icon className="w-6 h-6 text-blue-900 group-hover:text-white transition-colors" />
                  </div>
                  <span className="text-xs font-bold text-gray-400 uppercase tracking-wider">
                    {item.tag}
                  </span>
                </div>
                <h3 className="text-lg font-bold text-gray-900 mb-2">{item.title}</h3>
                <p className="text-sm text-gray-600 leading-relaxed">{item.desc}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── Team ─── */}
      <section className="py-24 px-6 bg-white">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16 space-y-4">
            <span className="inline-block bg-blue-100 text-blue-900 px-4 py-1.5 rounded-full text-sm font-semibold tracking-wide">
              Our Team
            </span>
            <h2 className="text-4xl md:text-5xl font-bold text-gray-900">
              The People Behind StelloVault
            </h2>
            <p className="text-lg text-gray-600 max-w-2xl mx-auto">
              A cross-disciplinary team of engineers, finance experts, and community builders working to democratize trade finance.
            </p>
          </div>

          <div className="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {teamMembers.map((member, idx) => (
              <TeamCard key={idx} member={member} />
            ))}
          </div>
        </div>
      </section>

      {/* ─── Roadmap ─── */}
      <Roadmap />

      {/* ─── CTA ─── */}
      <section className="py-24 px-6 bg-gradient-to-r from-blue-900 to-indigo-900 text-white">
        <div className="max-w-4xl mx-auto text-center space-y-8">
          <h2 className="text-4xl md:text-5xl font-bold">
            Join the Trade Finance Revolution
          </h2>
          <p className="text-xl text-blue-100/80 max-w-2xl mx-auto">
            Whether you&apos;re an SME seeking capital, an investor looking for real-world yield, or a developer building on Stellar — there&apos;s a place for you in StelloVault.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center pt-4">
            <a
              href="/contact"
              className="bg-white text-blue-900 px-10 py-4 rounded-full text-lg font-semibold hover:shadow-xl hover:scale-105 transition-all inline-block"
            >
              Get in Touch
            </a>
            <a
              href="https://github.com/stellovault"
              target="_blank"
              rel="noopener noreferrer"
              className="border-2 border-white/30 text-white px-10 py-4 rounded-full text-lg font-semibold hover:bg-white/10 transition-all inline-block"
            >
              View on GitHub
            </a>
          </div>
        </div>
      </section>

      <Footer />
    </main>
  )
}

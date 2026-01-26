'use client'

import React, { useState } from 'react'
import { ChevronRight, Lock, Zap, TrendingUp, CheckCircle2, ArrowUpRight, Wallet, LogOut } from 'lucide-react'

export default function Home() {
  const [isConnected, setIsConnected] = useState(false)
  const [walletAddress, setWalletAddress] = useState<string | null>(null)
  const [isWalletMenuOpen, setIsWalletMenuOpen] = useState(false)

  const handleWalletConnect = async () => {
    const mockAddress = 'G' + Math.random().toString(16).slice(2, 54).toUpperCase()
    setWalletAddress(mockAddress)
    setIsConnected(true)
    setIsWalletMenuOpen(false)
  }

  const handleWalletDisconnect = () => {
    setWalletAddress(null)
    setIsConnected(false)
  }

  const shortenAddress = (addr: string) => {
    return `${addr.slice(0, 6)}...${addr.slice(-4)}`
  }
  return (
    <div className="w-full min-h-screen bg-white">
      {/* Navigation */}
      <nav className="fixed w-full top-0 z-50 bg-white/95 backdrop-blur supports-[backdrop-filter]:bg-white/60 border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="text-2xl font-bold text-blue-900 flex items-center gap-2">
            <div className="w-8 h-8 bg-blue-900 rounded-lg flex items-center justify-center">
              <span className="text-white text-sm font-bold">SV</span>
            </div>
            StelloVault
          </div>
          <div className="hidden md:flex gap-8 text-sm text-gray-600">
            <a href="#features" className="hover:text-gray-900 transition">Features</a>
            <a href="#innovation" className="hover:text-gray-900 transition">Innovation</a>
            <a href="#impact" className="hover:text-gray-900 transition">Impact</a>
          </div>
          
          {/* Wallet Connect Button */}
          <div className="relative">
            {isConnected ? (
              <button
                onClick={() => setIsWalletMenuOpen(!isWalletMenuOpen)}
                className="flex items-center gap-2 bg-blue-100 text-blue-900 px-4 py-2 rounded-full text-sm font-medium hover:bg-blue-200 transition-colors"
              >
                <Wallet className="w-4 h-4" />
                {shortenAddress(walletAddress || '')}
              </button>
            ) : (
              <button
                onClick={() => setIsWalletMenuOpen(!isWalletMenuOpen)}
                className="flex items-center gap-2 bg-blue-900 text-white px-6 py-2 rounded-full text-sm font-medium hover:shadow-lg hover:scale-105 transition-all"
              >
                <Wallet className="w-4 h-4" />
                Connect Wallet
              </button>
            )}

            {isWalletMenuOpen && (
              <div className="absolute right-0 mt-2 w-64 bg-white rounded-xl shadow-xl border border-gray-200 z-50 overflow-hidden">
                <div className="p-4 border-b border-gray-100">
                  <h3 className="font-semibold text-gray-900 mb-3">Connect Your Wallet</h3>
                  <p className="text-xs text-gray-600 mb-4">
                    Select a Stellar wallet provider to sign in
                  </p>
                </div>

                <div className="p-3 space-y-2">
                  <button
                    onClick={handleWalletConnect}
                    className="w-full text-left px-4 py-3 rounded-lg hover:bg-blue-50 transition-colors border border-gray-200 hover:border-blue-300"
                  >
                    <div className="font-medium text-sm text-gray-900">Freighter</div>
                    <div className="text-xs text-gray-600">Stellar native wallet</div>
                  </button>

                  <button
                    onClick={handleWalletConnect}
                    className="w-full text-left px-4 py-3 rounded-lg hover:bg-blue-50 transition-colors border border-gray-200 hover:border-blue-300"
                  >
                    <div className="font-medium text-sm text-gray-900">Albedo</div>
                    <div className="text-xs text-gray-600">Web-based Stellar wallet</div>
                  </button>

                  <button
                    onClick={handleWalletConnect}
                    className="w-full text-left px-4 py-3 rounded-lg hover:bg-blue-50 transition-colors border border-gray-200 hover:border-blue-300"
                  >
                    <div className="font-medium text-sm text-gray-900">Rabet</div>
                    <div className="text-xs text-gray-600">Stellar browser extension</div>
                  </button>
                </div>

                {isConnected && (
                  <div className="p-3 border-t border-gray-100">
                    <button
                      onClick={handleWalletDisconnect}
                      className="w-full flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium text-gray-900 hover:bg-gray-50 transition-colors rounded-lg"
                    >
                      <LogOut className="w-4 h-4" />
                      Disconnect
                    </button>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="pt-32 pb-20 px-6 text-center bg-gradient-to-br from-white via-white to-gray-50">
        <div className="max-w-4xl mx-auto">
          <span className="inline-block bg-blue-100 text-blue-900 px-4 py-1 rounded-full text-sm font-medium mb-6">
            The Future of Trade Finance
          </span>
          <h1 className="text-5xl md:text-6xl font-bold text-gray-900 mb-6 leading-tight">
            Unlock Instant Cross-Border <span className="text-blue-900">Liquidity</span>
          </h1>
          <p className="text-xl text-gray-600 mb-8 max-w-2xl mx-auto leading-relaxed">
            Close the $100B+ trade finance gap. Tokenize real-world assets on Stellar, access instant liquidity, and transform how SMEs do business globally.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center pt-4">
            <button
              onClick={() => setIsWalletMenuOpen(true)}
              className="bg-blue-900 text-white px-8 py-4 rounded-full text-lg font-semibold hover:shadow-xl hover:scale-105 transition-all flex items-center justify-center gap-2 group"
            >
              Connect Wallet
              <ArrowUpRight className="w-5 h-5 group-hover:translate-x-1 group-hover:-translate-y-1 transition-transform" />
            </button>
            <button className="border-2 border-blue-900 text-blue-900 px-8 py-4 rounded-full text-lg font-semibold hover:bg-blue-50 transition-all flex items-center justify-center gap-2">
              Watch Demo
              <ChevronRight className="w-5 h-5" />
            </button>
          </div>

          <div className="pt-8 text-sm text-blue-500">
            ✓ No intermediaries • ✓ Low fees • ✓ Fast settlement
          </div>
        </div>

      
      </section>

      {/* Features Grid */}
      <section id="features" className="py-24 px-6 bg-white">
        <div className="max-w-6xl mx-auto">
          <div className="text-center space-y-4 mb-20">
            <h2 className="text-4xl md:text-5xl font-bold text-gray-900">Core Innovations</h2>
            <p className="text-lg text-gray-600 max-w-2xl mx-auto">
              Purpose-built infrastructure for trade finance on blockchain
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {/* Feature 1 */}
            <div className="group p-8 rounded-2xl border border-gray-200 hover:border-blue-300 bg-blue-900 hover:bg-blue-50 transition-all duration-300">
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-6 group-hover:bg-blue-200 transition-colors">
                <Lock className="w-6 h-6 text-blue-900" />
              </div>
              <h3 className="text-xl font-bold text-foreground mb-3  hover:text-blue-900">Collateral Tokenization</h3>
              <p className="text-foreground/70 leading-relaxed hover:text-gray-900">
                Real-world assets become fractional, traceable Stellar tokens with embedded metadata. Enable fractional ownership and instant marketability.
              </p>
            </div>

            {/* Feature 2 */}
            <div className="group p-8 rounded-2xl border border-gray-200 hover:border-blue-300 bg-blue-900 hover:bg-blue-50 transition-all duration-300">
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-6 group-hover:bg-blue-200 transition-colors">
                <Zap className="w-6 h-6 text-blue-900" />
              </div>
              <h3 className="text-xl font-bold text-foreground mb-3hover:text-blue-900">Automated Escrows</h3>
              <p className="text-foreground/70 leading-relaxed  hover:text-gray-900">
                Multi-signature escrows managed by Soroban smart contracts. Conditional release triggered by shipment verification oracles and IoT integrations.
              </p>
            </div>

            {/* Feature 3 */}
            <div className="group p-8 rounded-2xl border border-gray-200 hover:border-blue-300 bg-blue-900 hover:bg-blue-50 transition-all duration-300">
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-6 group-hover:bg-blue-200 transition-colors">
                <TrendingUp className="w-6 h-6 text-blue-900" />
              </div>
              <h3 className="text-xl font-bold text-foreground mb-3  hover:text-blue-900">Dynamic Financing</h3>
              <p className="text-foreground/70 leading-relaxed  hover:text-gray-900">
                Algorithmic loans based on on-chain transaction history and collateral utilization. Transparent risk assessment and instant approvals.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* The Problem Section */}
      <section id="impact" className="py-24 px-6 bg-gray-50">
        <div className="max-w-5xl mx-auto">
          <div className="grid md:grid-cols-2 gap-16 items-center">
            <div className="space-y-8">
              <h2 className="text-4xl font-bold text-gray-900">The Trade Finance Crisis</h2>
              
              <div className="space-y-6">
                <div className="flex gap-4">
                  <div className="w-6 h-6 rounded-full bg-blue-100 shrink-0 flex items-center justify-center mt-1">
                    <div className="w-2 h-2 bg-blue-900 rounded-full"></div>
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-1">$100–120B Annual Gap</h4>
                    <p className="text-gray-600">Trade finance shortfall disproportionately affecting SMEs across emerging markets, particularly Africa.</p>
                  </div>
                </div>

                <div className="flex gap-4">
                  <div className="w-6 h-6 rounded-full bg-blue-100 shrink-0 flex items-center justify-center mt-1">
                    <div className="w-2 h-2 bg-blue-900 rounded-full"></div>
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-1">90% of Businesses Are SMEs</h4>
                    <p className="text-gray-600">Yet they receive only a fraction of traditional trade finance support, limiting their growth potential.</p>
                  </div>
                </div>

                <div className="flex gap-4">
                  <div className="w-6 h-6 rounded-full bg-blue-100 shrink-0 flex items-center justify-center mt-1">
                    <div className="w-2 h-2 bg-blue-900 rounded-full"></div>
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-1">$100B+ Unrealized Trade</h4>
                    <p className="text-gray-600">Potential exports and intra-African trade under AfCFTA remain untapped due to financing barriers.</p>
                  </div>
                </div>
              </div>
            </div>

            <div className="space-y-8">
              <h2 className="text-4xl font-bold text-gray-900">Our Solution</h2>
              
              <div className="space-y-6">
                <div className="flex gap-4">
                  <div className="w-6 h-6 rounded-full bg-blue-100 shrink-0 flex items-center justify-center mt-1">
                    <CheckCircle2 className="w-4 h-4 text-blue-500" />
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-1 ">50% Cost Reduction</h4>
                    <p className="text-blue-900">Eliminate intermediaries using Stellar's low-cost settlements and Soroban smart contracts.</p>
                  </div>
                </div>

                <div className="flex gap-4">
                  <div className="w-6 h-6 rounded-full bg-blue-100 shrink-0 flex items-center justify-center mt-1">
                    <CheckCircle2 className="w-4 h-4 text-blue-500" />
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-1 ">Fractional Ownership</h4>
                    <p className="text-blue-900">Enable investors to own portions of real assets, democratizing access to high-value collateral.</p>
                  </div>
                </div>

                <div className="flex gap-4">
                  <div className="w-6 h-6 rounded-full bg-blue-100 shrink-0 flex items-center justify-center mt-1">
                    <CheckCircle2 className="w-4 h-4 text-blue-500" />
                  </div>
                  <div>
                    <h4 className="font-semibold text-gray-900 mb-1 ">Inclusive Trade Growth</h4>
                    <p className="text-blue-900">Foster intra-African commerce and SME empowerment through accessible, transparent financing.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section id="innovation" className="py-24 px-6 bg-white">
        <div className="max-w-5xl mx-auto">
          <div className="text-center space-y-4 mb-20">
            <h2 className="text-4xl md:text-5xl font-bold text-blue-900">How StelloVault Works</h2>
            <p className="text-lg text-gray-900">
              A seamless workflow powered by blockchain and smart contracts
            </p>
          </div>

          <div className="space-y-4">
            {[
              { step: 1, title: "Tokenize Your Assets", desc: "SMEs upload asset details (invoices, commodities, receivables). StelloVault verifies and tokenizes as Stellar assets." },
              { step: 2, title: "Set Up Smart Contracts", desc: "Soroban contracts configure multi-sig escrows with conditional release logic tied to fulfillment oracles." },
              { step: 3, title: "Access Liquidity", desc: "Investors bid on your collateral. Secure financing with minimal intermediaries and instant settlement." },
              { step: 4, title: "Unlock Growth", desc: "Use capital for inventory, expansion, or operations. Repay terms set transparently on-chain." },
            ].map((item, idx) => (
              <div key={idx} className="relative">
                <div className="flex gap-6">
                  <div className="relative flex flex-col items-center">
                    <div className="w-10 h-10 rounded-full bg-blue-900 text-white flex items-center justify-center font-bold text-lg relative z-10">
                      {item.step}
                    </div>
                    {idx < 3 && (
                      <div className="w-1 h-16 bg-gradient-to-b from-blue-300 to-blue-100 mt-2"></div>
                    )}
                  </div>
                  <div className="pb-8 pt-2 flex-1">
                    <h3 className="text-xl font-bold text-gray-900 mb-2">{item.title}</h3>
                    <p className="text-gray-600">{item.desc}</p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Risk Scoring */}
      <section className="py-24 px-6 bg-blue-50">
        <div className="max-w-4xl mx-auto text-center space-y-8">
          <h2 className="text-4xl font-bold text-gray-900">Intelligent Risk Scoring</h2>
          <p className="text-lg text-gray-600">
            Our backend harnesses transaction history and collateral analytics to deliver fair, transparent creditworthiness assessment. No bias. Pure data.
          </p>

          <div className="grid md:grid-cols-3 gap-6 pt-8">
            <div className="bg-white rounded-xl p-6 border border-gray-200">
              <div className="text-3xl font-bold text-blue-900 mb-2">Real-time</div>
              <p className="text-gray-600">Instant creditworthiness scores based on on-chain behavior</p>
            </div>
            <div className="bg-white rounded-xl p-6 border border-gray-200">
              <div className="text-3xl font-bold text-blue-900 mb-2">Transparent</div>
              <p className="text-gray-600">Every factor auditable and explainable to borrowers</p>
            </div>
            <div className="bg-white rounded-xl p-6 border border-gray-200">
              <div className="text-3xl font-bold text-blue-900 mb-2">Fair</div>
              <p className="text-gray-600">Algorithm-driven, free from traditional lending bias</p>
            </div>
          </div>
        </div>
      </section>

      {/* Governance */}
      <section className="py-24 px-6 bg-white">
        <div className="max-w-4xl mx-auto">
          <div className="bg-gradient-to-br from-blue-100 to-cyan-50 rounded-3xl p-12 text-center space-y-6">
            <h2 className="text-4xl font-bold text-gray-900">Community-Driven Governance</h2>
            <p className="text-lg text-gray-600">
              StelloVault stakeholders use quadratic voting to decide which collateral types are accepted on the platform. True decentralization. Aligned incentives.
            </p>
            <p className="text-sm text-gray-500">
              Your voice shapes the future of trade finance.
            </p>
          </div>
        </div>
      </section>

      {/* Why Stellar & Soroban */}
      <section className="py-24 px-6 bg-gray-50">
        <div className="max-w-5xl mx-auto">
          <h2 className="text-4xl font-bold text-gray-900 text-center mb-16">Built on Best-in-Class Technology</h2>

          <div className="grid md:grid-cols-2 gap-12">
            <div className="space-y-6">
              <div className="flex gap-4">
                <div className="flex-shrink-0">
                  <div className="flex items-center justify-center h-12 w-12 rounded-md bg-blue-100">
                    <ArrowUpRight className="h-6 w-6 text-blue-900" />
                  </div>
                </div>
                <div>
                  <h3 className="text-xl font-bold text-gray-900 mb-2">Stellar Protocol</h3>
                  <p className="text-gray-600">
                    Low-cost, fast cross-border settlements. Native multi-asset support perfect for tokenizing real-world assets and enabling global liquidity.
                  </p>
                </div>
              </div>
              
              <div className="flex gap-4">
                <div className="flex-shrink-0">
                  <div className="flex items-center justify-center h-12 w-12 rounded-md bg-cyan-100">
                    <Lock className="h-6 w-6 text-cyan-900" />
                  </div>
                </div>
                <div>
                  <h3 className="text-xl font-bold text-gray-900 mb-2">Soroban Smart Contracts</h3>
                  <p className="text-gray-600">
                    Rust-based, auditable, and production-hardened. Enables programmable, trustless escrows with multi-signature verification and conditional logic.
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-white rounded-2xl p-8 border border-gray-200">
              <h3 className="text-2xl font-bold text-gray-900 mb-6">Platform Benefits</h3>
              <ul className="space-y-4">
                {[
                  "Sub-second settlement times",
                  "Fractional asset ownership",
                  "Multi-signature security",
                  "Programmable payments",
                  "Real-time transparency",
                  "Compliance-ready architecture",
                ].map((benefit, idx) => (
                  <li key={idx} className="flex gap-3 items-start">
                    <CheckCircle2 className="w-5 h-5 text-blue-900 flex-shrink-0 mt-0.5" />
                    <span className="text-gray-700">{benefit}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-24 px-6 bg-gradient-to-r from-blue-50 to-cyan-50">
        <div className="max-w-4xl mx-auto text-center space-y-8">
          <h2 className="text-4xl md:text-5xl font-bold text-gray-900">Ready to Unlock Trade Finance?</h2>
          <p className="text-xl text-gray-600">
            Join the revolution. Be among the first SMEs and investors reshaping global trade.
          </p>
          
          <div className="flex flex-col sm:flex-row gap-4 justify-center pt-4">
            <button
              onClick={() => setIsWalletMenuOpen(true)}
              className="bg-blue-900 text-white px-10 py-4 rounded-full text-lg font-semibold hover:shadow-xl hover:scale-105 transition-all"
            >
              Connect Wallet & Start Trading
            </button>
            <button className="border-2 border-blue-900 text-blue-900 px-10 py-4 rounded-full text-lg font-semibold hover:bg-blue-50 transition-all">
              Join Community
            </button>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="bg-gray-900 text-white py-12 px-6">
        <div className="max-w-6xl mx-auto">
          <div className="grid md:grid-cols-4 gap-8 pb-12 border-b border-white/10">
            <div>
              <h3 className="font-bold text-lg mb-4 flex items-center gap-2">
                <div className="w-8 h-8 bg-white/20 rounded-lg flex items-center justify-center">
                  <span className="text-sm font-bold">SV</span>
                </div>
                StelloVault
              </h3>
              <p className="text-white/70 text-sm">
                Tokenizing trade. Financing futures.
              </p>
            </div>
            <div>
              <h4 className="font-semibold mb-4">Product</h4>
              <ul className="space-y-2 text-sm text-white/70">
                <li><a href="#" className="hover:text-white transition">Features</a></li>
                <li><a href="#" className="hover:text-white transition">Pricing</a></li>
                <li><a href="#" className="hover:text-white transition">Security</a></li>
              </ul>
            </div>
            <div>
              <h4 className="font-semibold mb-4">Company</h4>
              <ul className="space-y-2 text-sm text-white/70">
                <li><a href="#" className="hover:text-white transition">About</a></li>
                <li><a href="#" className="hover:text-white transition">Blog</a></li>
                <li><a href="#" className="hover:text-white transition">Careers</a></li>
              </ul>
            </div>
            <div>
              <h4 className="font-semibold mb-4">Legal</h4>
              <ul className="space-y-2 text-sm text-white/70">
                <li><a href="#" className="hover:text-white transition">Privacy</a></li>
                <li><a href="#" className="hover:text-white transition">Terms</a></li>
                <li><a href="#" className="hover:text-white transition">Contact</a></li>
              </ul>
            </div>
          </div>

          <div className="flex flex-col md:flex-row justify-between items-center pt-8 text-sm text-white/60">
            <p>&copy; 2025 StelloVault. All rights reserved.</p>
            <div className="flex gap-6 mt-4 md:mt-0">
              <a href="#" className="hover:text-white transition">Twitter</a>
              <a href="#" className="hover:text-white transition">LinkedIn</a>
              <a href="#" className="hover:text-white transition">GitHub</a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  )
}

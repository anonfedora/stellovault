'use client'

import React, { useState } from 'react'
import { Wallet, LogOut, Menu, X, Loader2 } from 'lucide-react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useTransactionStatus } from '@/contexts/TransactionStatusProvider'
import { TransactionHistoryButton } from '@/components/transactions/TransactionHistoryDrawer'

export function Navbar() {
  const pathname = usePathname()
  const [isConnected, setIsConnected] = useState(false)
  const [walletAddress, setWalletAddress] = useState<string | null>(null)
  const [isWalletMenuOpen, setIsWalletMenuOpen] = useState(false)
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)
  const { pendingCount } = useTransactionStatus()

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

  const isActive = (path: string) => {
    if (path === '/') return pathname === '/'
    if (path.startsWith('/#')) return pathname === '/'
    return pathname === path
  }

  const navLinks = [
    { href: '/#features', label: 'Features' },
    { href: '/#innovation', label: 'Innovation' },
    { href: '/#impact', label: 'Impact' },
    { href: '/about', label: 'About' },
    { href: '/contact', label: 'Contact' },
    { href: '/governance', label: 'Governance' },
  ]

  return (
    <nav className="fixed w-full top-0 z-50 bg-white/95 backdrop-blur supports-[backdrop-filter]:bg-white/60 border-b border-gray-200">
      <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
        <Link href="/" className="text-2xl font-bold text-blue-900 flex items-center gap-2">
          <div className="w-8 h-8 bg-blue-900 rounded-lg flex items-center justify-center">
            <span className="text-white text-sm font-bold">SV</span>
          </div>
          StelloVault
        </Link>
        
        <div className="hidden md:flex gap-8 text-sm items-center">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className={`transition-colors duration-200 ${
                isActive(link.href)
                  ? 'text-blue-900 font-bold border-b-2 border-blue-900 pb-1'
                  : 'text-gray-600 hover:text-blue-900'
              }`}
            >
              {link.label}
            </Link>
          ))}
        </div>
        
        <div className="flex items-center gap-4">
          {/* Pending Transaction Indicator */}
          {pendingCount > 0 && (
            <div className="flex items-center gap-2 px-3 py-1.5 bg-yellow-100 text-yellow-800 rounded-full text-sm font-medium">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>{pendingCount} Pending</span>
            </div>
          )}
          
          {/* Transaction History Button */}
          <TransactionHistoryButton />
          
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
                  <h3 className="font-semibold text-gray-900 mb-1">Connect Wallet</h3>
                  <p className="text-xs text-gray-500">
                    Select a Stellar wallet provider
                  </p>
                </div>

                <div className="p-2 space-y-1">
                  {['Freighter', 'Albedo', 'Rabet'].map((provider) => (
                    <button
                      key={provider}
                      onClick={handleWalletConnect}
                      className="w-full text-left px-4 py-2 rounded-lg hover:bg-blue-50 transition-colors"
                    >
                      <div className="font-medium text-sm text-gray-900">{provider}</div>
                      <div className="text-xs text-gray-500">Stellar wallet</div>
                    </button>
                  ))}
                </div>

                {isConnected && (
                  <div className="p-2 border-t border-gray-100">
                    <button
                      onClick={handleWalletDisconnect}
                      className="w-full flex items-center gap-2 px-4 py-2 text-sm font-medium text-red-600 hover:bg-red-50 transition-colors rounded-lg"
                    >
                      <LogOut className="w-4 h-4" />
                      Disconnect
                    </button>
                  </div>
                )}
              </div>
            )}
          </div>
          
          <button 
            className="md:hidden p-2 text-gray-600"
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
          >
            {isMobileMenuOpen ? <X /> : <Menu />}
          </button>
        </div>
      </div>

      {/* Mobile Menu */}
      {isMobileMenuOpen && (
        <div className="md:hidden bg-white border-b border-gray-200 px-6 py-4 space-y-4">
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              onClick={() => setIsMobileMenuOpen(false)}
              className={`block py-2 transition-colors duration-200 ${
                isActive(link.href)
                  ? 'text-blue-900 font-bold bg-blue-50 px-3 rounded-lg'
                  : 'text-gray-600 hover:text-blue-900'
              }`}
            >
              {link.label}
            </Link>
          ))}
        </div>
      )}
    </nav>
  )
}


import React from 'react'
import Link from 'next/link'
import { Github, Twitter, Send, MessageSquare } from 'lucide-react'

export function Footer() {
  return (
    <footer className="bg-gray-900 text-white py-16 px-6">
      <div className="max-w-7xl mx-auto">
        <div className="grid md:grid-cols-4 gap-12 pb-12 border-b border-white/10">
          <div className="space-y-4">
            <Link href="/" className="text-2xl font-bold flex items-center gap-2">
              <div className="w-8 h-8 bg-white/20 rounded-lg flex items-center justify-center">
                <span className="text-sm font-bold">SV</span>
              </div>
              StelloVault
            </Link>
            <p className="text-gray-400 text-sm leading-relaxed">
              Tokenizing trade finance. Powering SMEs with instant on-chain liquidity on the Stellar network.
            </p>
          </div>
          
          <div>
            <h4 className="font-semibold mb-6">Product</h4>
            <ul className="space-y-4 text-sm text-gray-400">
              <li><Link href="/#features" className="hover:text-white transition">Features</Link></li>
              <li><Link href="/#innovation" className="hover:text-white transition">How it Works</Link></li>
              <li><Link href="/dashboard" className="hover:text-white transition">Dashboard</Link></li>
            </ul>
          </div>
          
          <div>
            <h4 className="font-semibold mb-6">Company</h4>
            <ul className="space-y-4 text-sm text-gray-400">
              <li><Link href="/about" className="hover:text-white transition">About Us</Link></li>
              <li><Link href="/blog" className="hover:text-white transition">Blog</Link></li>
              <li><Link href="/careers" className="hover:text-white transition">Careers</Link></li>
            </ul>
          </div>
          
          <div>
            <h4 className="font-semibold mb-6">Support</h4>
            <ul className="space-y-4 text-sm text-gray-400">
              <li><Link href="/contact" className="hover:text-white transition">Contact Us</Link></li>
              <li><Link href="/docs" className="hover:text-white transition">Documentation</Link></li>
              <li><Link href="/faq" className="hover:text-white transition">FAQ</Link></li>
            </ul>
          </div>
        </div>

        <div className="flex flex-col md:flex-row justify-between items-center pt-8 text-sm text-gray-500">
          <div className="mb-4 md:mb-0">
            &copy; {new Date().getFullYear()} StelloVault. Built on Stellar.
          </div>
          <div className="flex gap-6">
            <Link href="https://github.com/stellovault" target="_blank" className="hover:text-white transition">
              <Github className="w-5 h-5" />
            </Link>
            <Link href="https://twitter.com/stellovault" target="_blank" className="hover:text-white transition">
              <Twitter className="w-5 h-5" />
            </Link>
            <Link href="https://discord.gg/stellovault" target="_blank" className="hover:text-white transition">
              <MessageSquare className="w-5 h-5" />
            </Link>
            <Link href="https://t.me/stellovault" target="_blank" className="hover:text-white transition">
              <Send className="w-5 h-5" />
            </Link>
          </div>
        </div>
      </div>
    </footer>
  )
}

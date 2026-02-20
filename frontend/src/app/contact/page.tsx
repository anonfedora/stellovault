
import React from 'react'
import { Navbar } from '@/components/layout/Navbar'
import { Footer } from '@/components/layout/Footer'
import { ContactSection } from '@/components/contact/ContactSection'
import { FAQ } from '@/components/contact/FAQ'
import { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'Contact Us | StelloVault',
  description: 'Reach out to the StelloVault team for partnerships, support, or bug reports. Build the future of trade finance with us.',
}

export default function ContactPage() {
  return (
    <main className="min-h-screen bg-white">
      <Navbar />
      
      {/* Hero Header */}
      <div className="pt-32 pb-16 px-6 text-center bg-blue-900 text-white relative overflow-hidden">
        {/* Abstract background blobs */}
        <div className="absolute top-0 left-0 w-full h-full overflow-hidden z-0 opacity-20 pointer-events-none">
          <div className="absolute -top-[10%] -left-[5%] w-[40%] h-[60%] bg-blue-400 blur-[120px] rounded-full animate-pulse" />
          <div className="absolute top-[40%] -right-[10%] w-[50%] h-[50%] bg-cyan-400 blur-[130px] rounded-full" />
        </div>
        
        <div className="max-w-4xl mx-auto relative z-10">
          <h1 className="text-4xl md:text-6xl font-bold mb-6">Contact Our Team</h1>
          <p className="text-xl text-blue-100/80 max-w-2xl mx-auto">
            Connecting global liquidity to real-world trade. We're here to support your journey into on-chain financing.
          </p>
        </div>
      </div>

      <ContactSection />
      
      <div className="bg-gray-50/50">
        <FAQ />
      </div>

      <Footer />
    </main>
  )
}

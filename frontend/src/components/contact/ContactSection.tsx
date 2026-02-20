
'use client'

import React, { useState } from 'react'
import { Send, Github, Twitter, MessageSquare, SendIcon, CheckCircle2 } from 'lucide-react'

export function ContactSection() {
  const [formState, setFormState] = useState<'idle' | 'loading' | 'success'>('idle')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setFormState('loading')
    // Simulate API call
    setTimeout(() => {
      setFormState('success')
    }, 1500)
  }

  return (
    <section className="py-24 px-6 bg-gradient-to-b from-white to-blue-50/50">
      <div className="max-w-7xl mx-auto">
        <div className="grid lg:grid-cols-2 gap-16 items-start">
          {/* Contact Info & Socials */}
          <div className="space-y-12">
            <div>
              <h2 className="text-4xl md:text-5xl font-bold text-gray-900 mb-6 leading-tight">
                Let's Build the Future of <span className="text-blue-900">Trade Finance</span> Together
              </h2>
              <p className="text-xl text-gray-600 leading-relaxed max-w-lg">
                Have questions about our protocol? Want to partner or join our oracle network? Reach out and our team will get back to you within 24 hours.
              </p>
            </div>

            <div className="grid sm:grid-cols-2 gap-8">
              {[
                { icon: Github, label: 'GitHub', username: '@stellovault', link: 'https://github.com/stellovault', color: 'hover:text-black hover:bg-gray-100' },
                { icon: Twitter, label: 'Twitter / X', username: '@StelloVault', link: 'https://twitter.com/stellovault', color: 'hover:text-[#1DA1F2] hover:bg-blue-50' },
                { icon: MessageSquare, label: 'Discord', username: 'StelloVault Community', link: 'https://discord.gg/stellovault', color: 'hover:text-[#5865F2] hover:bg-indigo-50' },
                { icon: SendIcon, label: 'Telegram', username: '@StelloVaultInternal', link: 'https://t.me/stellovault', color: 'hover:text-[#0088cc] hover:bg-sky-50' },
              ].map((social) => (
                <a
                  key={social.label}
                  href={social.link}
                  target="_blank"
                  rel="noopener noreferrer"
                  className={`flex items-center gap-4 p-4 rounded-2xl border border-gray-100 bg-white transition-all duration-300 group ${social.color}`}
                >
                  <div className="w-12 h-12 rounded-xl bg-gray-50 flex items-center justify-center group-hover:bg-transparent transition-colors">
                    <social.icon className="w-6 h-6" />
                  </div>
                  <div>
                    <div className="text-sm font-medium text-gray-400">{social.label}</div>
                    <div className="font-semibold text-gray-900">{social.username}</div>
                  </div>
                </a>
              ))}
            </div>

            <div className="p-6 rounded-2xl bg-blue-900 text-white shadow-xl">
              <h4 className="text-lg font-bold mb-2">Office Headquarters</h4>
              <p className="text-blue-100/80 mb-4">Port Louis, Mauritius • Digital First Team</p>
              <div className="text-sm space-y-1">
                <p>Partnerships: partners@stellovault.io</p>
                <p>Support: support@stellovault.io</p>
              </div>
            </div>
          </div>

          {/* Contact Form */}
          <div className="bg-white rounded-3xl p-8 md:p-12 shadow-2xl border border-gray-100">
            {formState === 'success' ? (
              <div className="h-[500px] flex flex-col items-center justify-center text-center space-y-6">
                <div className="w-20 h-20 bg-green-100 rounded-full flex items-center justify-center">
                  <CheckCircle2 className="w-10 h-10 text-green-600" />
                </div>
                <h3 className="text-2xl font-bold text-gray-900">Message Sent!</h3>
                <p className="text-gray-600 max-w-xs mx-auto">
                  Thank you for reaching out. Our team will review your inquiry and get back to you shortly.
                </p>
                <button 
                  onClick={() => setFormState('idle')}
                  className="text-blue-900 font-semibold hover:underline"
                >
                  Send another message
                </button>
              </div>
            ) : (
              <form onSubmit={handleSubmit} className="space-y-6">
                <div className="grid md:grid-cols-2 gap-6">
                  <div className="space-y-2">
                    <label className="text-sm font-semibold text-gray-700 ml-1">Full Name</label>
                    <input
                      required
                      type="text"
                      className="w-full px-5 py-4 rounded-xl border border-gray-200 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 outline-none transition-all text-black"
                      placeholder="John Doe"
                    />
                  </div>
                  <div className="space-y-2">
                    <label className="text-sm font-semibold text-gray-700 ml-1">Email Address</label>
                    <input
                      required
                      type="email"
                      className="w-full px-5 py-4 rounded-xl border border-gray-200 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 outline-none transition-all text-black"
                      placeholder="john@company.com"
                    />
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-semibold text-gray-700 ml-1">Subject</label>
                  <select
                    className="w-full px-5 py-4 rounded-xl border border-gray-200 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 outline-none transition-all appearance-none bg-white text-black"
                  >
                    <option>General Inquiry</option>
                    <option>Partnership Proposal</option>
                    <option>Bug Report</option>
                    <option>Oracle Registration</option>
                  </select>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-semibold text-gray-700 ml-1">Message</label>
                  <textarea
                    required
                    rows={4}
                    className="w-full px-5 py-4 rounded-xl border border-gray-200 focus:border-blue-500 focus:ring-2 focus:ring-blue-100 outline-none transition-all resize-none text-black"
                    placeholder="Tell us how we can help..."
                  />
                </div>

                <button
                  type="submit"
                  disabled={formState === 'loading'}
                  className="w-full py-5 bg-blue-900 text-white rounded-xl font-bold text-lg hover:shadow-xl hover:scale-[1.02] active:scale-100 disabled:opacity-70 disabled:scale-100 transition-all flex items-center justify-center gap-3"
                >
                  {formState === 'loading' ? (
                    <div className="w-6 h-6 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  ) : (
                    <>
                      Send Message
                      <Send className="w-5 h-5" />
                    </>
                  )}
                </button>

                <p className="text-center text-xs text-gray-400">
                  By submitting this form, you agree to our Privacy Policy.
                </p>
              </form>
            )}
          </div>
        </div>
      </div>
    </section>
  )
}

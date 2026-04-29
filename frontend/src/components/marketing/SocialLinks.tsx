import React from 'react'
import { Github, Twitter, MessageSquare, Send } from 'lucide-react'

interface SocialLink {
  label: string
  href: string
  icon: React.ReactNode
  description: string
}

const LINKS: SocialLink[] = [
  {
    label: 'GitHub',
    href: 'https://github.com/anonfedora/stellovault',
    icon: <Github className="h-5 w-5" />,
    description: 'Browse the source code',
  },
  {
    label: 'Twitter / X',
    href: 'https://twitter.com/stellovault',
    icon: <Twitter className="h-5 w-5" />,
    description: 'Follow for updates',
  },
  {
    label: 'Discord',
    href: 'https://discord.gg/stellovault',
    icon: <MessageSquare className="h-5 w-5" />,
    description: 'Join the community',
  },
  {
    label: 'Telegram',
    href: 'https://t.me/stellovault',
    icon: <Send className="h-5 w-5" />,
    description: 'Chat with the team',
  },
]

export function SocialLinks() {
  return (
    <div className="grid grid-cols-2 gap-4">
      {LINKS.map(link => (
        <a
          key={link.label}
          href={link.href}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-3 p-4 rounded-xl border border-gray-200 hover:border-blue-900 hover:bg-blue-50 transition-all group"
        >
          <span className="text-gray-500 group-hover:text-blue-900 transition-colors">
            {link.icon}
          </span>
          <div>
            <p className="text-sm font-semibold text-gray-900">{link.label}</p>
            <p className="text-xs text-gray-400">{link.description}</p>
          </div>
        </a>
      ))}
    </div>
  )
}

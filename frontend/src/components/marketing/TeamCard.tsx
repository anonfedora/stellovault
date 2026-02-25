import React from 'react'
import { Github, Twitter } from 'lucide-react'
import Link from 'next/link'

export interface TeamMember {
  name: string
  role: string
  bio: string
  avatar: string
  github?: string
  twitter?: string
}

export function TeamCard({ member }: { member: TeamMember }) {
  return (
    <div className="group relative bg-white rounded-2xl border border-gray-100 p-6 hover:border-blue-200 hover:shadow-xl transition-all duration-300">
      {/* Avatar */}
      <div className="flex items-center gap-4 mb-4">
        <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-blue-900 to-indigo-700 flex items-center justify-center text-white text-xl font-bold shadow-md group-hover:scale-105 transition-transform">
          {member.avatar}
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="text-lg font-bold text-gray-900 truncate">{member.name}</h3>
          <p className="text-sm font-semibold text-blue-700">{member.role}</p>
        </div>
      </div>

      {/* Bio */}
      <p className="text-sm text-gray-600 leading-relaxed mb-5">{member.bio}</p>

      {/* Social Links */}
      <div className="flex gap-3">
        {member.github && (
          <Link
            href={member.github}
            target="_blank"
            rel="noopener noreferrer"
            aria-label={`${member.name} GitHub profile`}
            className="w-9 h-9 rounded-lg bg-gray-50 hover:bg-gray-900 hover:text-white flex items-center justify-center text-gray-500 transition-all"
          >
            <Github className="w-4 h-4" />
          </Link>
        )}
        {member.twitter && (
          <Link
            href={member.twitter}
            target="_blank"
            rel="noopener noreferrer"
            aria-label={`${member.name} Twitter profile`}
            className="w-9 h-9 rounded-lg bg-gray-50 hover:bg-blue-500 hover:text-white flex items-center justify-center text-gray-500 transition-all"
          >
            <Twitter className="w-4 h-4" />
          </Link>
        )}
      </div>
    </div>
  )
}

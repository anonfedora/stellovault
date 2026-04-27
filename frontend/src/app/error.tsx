'use client'

import { useEffect } from 'react'
import { AlertTriangle, RefreshCw, Home } from 'lucide-react'
import Link from 'next/link'

interface Props {
  error: Error & { digest?: string }
  reset: () => void
}

export default function GlobalError({ error, reset }: Props) {
  useEffect(() => {
    console.error('[GlobalError] Unhandled error:', error)
  }, [error])

  return (
    <main className="min-h-screen bg-gray-50 flex items-center justify-center px-6">
      <div className="max-w-lg w-full text-center space-y-8">
        <div className="flex justify-center">
          <div className="bg-red-100 rounded-full p-6">
            <AlertTriangle className="h-16 w-16 text-red-600" />
          </div>
        </div>

        <div>
          <p className="text-6xl font-extrabold text-red-600 mb-2">500</p>
          <h1 className="text-2xl font-bold text-gray-900 mb-4">Something went wrong</h1>
          <p className="text-gray-500 leading-relaxed">
            An unexpected error occurred. Our team has been notified.
            {error.digest && (
              <span className="block mt-2 font-mono text-xs text-gray-400">
                Error ID: {error.digest}
              </span>
            )}
          </p>
        </div>

        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          <button
            onClick={reset}
            className="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-blue-900 text-white font-medium hover:bg-blue-800 transition-colors"
          >
            <RefreshCw className="h-4 w-4" />
            Try again
          </button>
          <Link
            href="/"
            className="inline-flex items-center gap-2 px-6 py-3 rounded-xl border border-gray-300 text-gray-700 font-medium hover:bg-gray-100 transition-colors"
          >
            <Home className="h-4 w-4" />
            Go home
          </Link>
        </div>
      </div>
    </main>
  )
}

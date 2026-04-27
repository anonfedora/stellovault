import Link from 'next/link'
import { FileQuestion, Home, ArrowLeft } from 'lucide-react'

export default function NotFound() {
  return (
    <main className="min-h-screen bg-gray-50 flex items-center justify-center px-6">
      <div className="max-w-lg w-full text-center space-y-8">
        <div className="flex justify-center">
          <div className="bg-blue-100 rounded-full p-6">
            <FileQuestion className="h-16 w-16 text-blue-900" />
          </div>
        </div>

        <div>
          <p className="text-6xl font-extrabold text-blue-900 mb-2">404</p>
          <h1 className="text-2xl font-bold text-gray-900 mb-4">Page not found</h1>
          <p className="text-gray-500 leading-relaxed">
            The page you&apos;re looking for doesn&apos;t exist or has been moved.
            Check the URL or head back to the dashboard.
          </p>
        </div>

        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          <Link
            href="/"
            className="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-blue-900 text-white font-medium hover:bg-blue-800 transition-colors"
          >
            <Home className="h-4 w-4" />
            Go to dashboard
          </Link>
          <Link
            href="javascript:history.back()"
            className="inline-flex items-center gap-2 px-6 py-3 rounded-xl border border-gray-300 text-gray-700 font-medium hover:bg-gray-100 transition-colors"
          >
            <ArrowLeft className="h-4 w-4" />
            Go back
          </Link>
        </div>
      </div>
    </main>
  )
}

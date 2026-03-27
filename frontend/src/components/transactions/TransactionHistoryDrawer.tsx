'use client'

import React from 'react'
import { X, ExternalLink, Clock, CheckCircle2, XCircle, Loader2, Trash2 } from 'lucide-react'
import { useTransactionStatus, Transaction, TransactionStatus } from '@/contexts/TransactionStatusProvider'

// Status icon component
function StatusIcon({ status }: { status: TransactionStatus }) {
  switch (status) {
    case 'success':
      return <CheckCircle2 className="w-5 h-5 text-green-500" />
    case 'failed':
      return <XCircle className="w-5 h-5 text-red-500" />
    case 'pending':
    case 'open':
      return <Loader2 className="w-5 h-5 text-yellow-500 animate-spin" />
    default:
      return <Clock className="w-5 h-5 text-gray-400" />
  }
}

// Status badge component
function StatusBadge({ status }: { status: TransactionStatus }) {
  const baseClasses = 'px-2 py-0.5 rounded-full text-xs font-medium'
  
  switch (status) {
    case 'success':
      return <span className={`${baseClasses} bg-green-100 text-green-700`}>Success</span>
    case 'failed':
      return <span className={`${baseClasses} bg-red-100 text-red-700`}>Failed</span>
    case 'pending':
    case 'open':
      return <span className={`${baseClasses} bg-yellow-100 text-yellow-700`}>Pending</span>
    default:
      return <span className={`${baseClasses} bg-gray-100 text-gray-600`}>Unknown</span>
  }
}

// Format timestamp
function formatTimestamp(timestamp: number): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)
  
  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  })
}

// Shorten transaction hash
function shortenHash(hash: string): string {
  if (!hash) return 'Pending...'
  return `${hash.slice(0, 8)}...${hash.slice(-4)}`
}

// Transaction item component
function TransactionItem({ transaction }: { transaction: Transaction }) {
  return (
    <div className="p-4 border-b border-gray-100 hover:bg-gray-50 transition-colors">
      <div className="flex items-start gap-3">
        <StatusIcon status={transaction.status} />
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between gap-2">
            <h4 className="font-medium text-sm text-gray-900 truncate">
              {transaction.description}
            </h4>
            <StatusBadge status={transaction.status} />
          </div>
          
          <div className="mt-1 flex items-center gap-2 text-xs text-gray-500">
            <span className="uppercase font-mono bg-gray-100 px-1.5 py-0.5 rounded">
              {transaction.type}
            </span>
            <span>{formatTimestamp(transaction.timestamp)}</span>
          </div>
          
          {transaction.hash && (
            <div className="mt-2 flex items-center gap-2">
              <code className="text-xs font-mono text-gray-600 bg-gray-100 px-2 py-1 rounded">
                {shortenHash(transaction.hash)}
              </code>
              {transaction.url && (
                <a
                  href={transaction.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-1 text-xs text-blue-600 hover:text-blue-800 hover:underline"
                >
                  <ExternalLink className="w-3 h-3" />
                  View
                </a>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

// Empty state component
function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
      <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4">
        <Clock className="w-8 h-8 text-gray-400" />
      </div>
      <h3 className="font-medium text-gray-900 mb-1">No transactions yet</h3>
      <p className="text-sm text-gray-500">
        Your transaction history will appear here
      </p>
    </div>
  )
}

// Main drawer component
export function TransactionHistoryDrawer() {
  const { transactions, isDrawerOpen, setDrawerOpen, clearHistory, pendingCount } = useTransactionStatus()

  if (!isDrawerOpen) return null

  return (
    <>
      {/* Backdrop */}
      <div 
        className="fixed inset-0 bg-black/30 z-40 transition-opacity"
        onClick={() => setDrawerOpen(false)}
      />
      
      {/* Drawer */}
      <div className="fixed right-0 top-0 h-full w-full max-w-md bg-white shadow-2xl z-50 flex flex-col animate-in slide-in-from-right duration-300">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200 bg-gray-50">
          <div className="flex items-center gap-3">
            <h2 className="text-lg font-semibold text-gray-900">Transaction History</h2>
            {pendingCount > 0 && (
              <span className="flex items-center gap-1 px-2 py-0.5 bg-yellow-100 text-yellow-700 rounded-full text-xs font-medium">
                <Loader2 className="w-3 h-3 animate-spin" />
                {pendingCount} pending
              </span>
            )}
          </div>
          <button
            onClick={() => setDrawerOpen(false)}
            className="p-2 hover:bg-gray-200 rounded-lg transition-colors"
            aria-label="Close drawer"
          >
            <X className="w-5 h-5 text-gray-500" />
          </button>
        </div>
        
        {/* Transaction list */}
        <div className="flex-1 overflow-y-auto">
          {transactions.length === 0 ? (
            <EmptyState />
          ) : (
            <div className="divide-y divide-gray-100">
              {transactions.map((transaction) => (
                <TransactionItem key={transaction.id} transaction={transaction} />
              ))}
            </div>
          )}
        </div>
        
        {/* Footer */}
        {transactions.length > 0 && (
          <div className="p-4 border-t border-gray-200 bg-gray-50">
            <button
              onClick={clearHistory}
              className="w-full flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium text-red-600 hover:bg-red-50 rounded-lg transition-colors"
            >
              <Trash2 className="w-4 h-4" />
              Clear History
            </button>
          </div>
        )}
      </div>
    </>
  )
}

// Button to open the drawer (for use in Navbar or elsewhere)
export function TransactionHistoryButton() {
  const { transactions, pendingCount, setDrawerOpen } = useTransactionStatus()

  return (
    <button
      onClick={() => setDrawerOpen(true)}
      className="relative flex items-center gap-2 px-3 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
      aria-label="View transaction history"
    >
      <Clock className="w-5 h-5" />
      <span className="hidden sm:inline">History</span>
      
      {pendingCount > 0 && (
        <span className="absolute -top-1 -right-1 flex items-center justify-center w-5 h-5 bg-yellow-500 text-white text-xs font-bold rounded-full">
          {pendingCount}
        </span>
      )}
      
      {transactions.length > 0 && pendingCount === 0 && (
        <span className="absolute -top-1 -right-1 flex items-center justify-center w-5 h-5 bg-blue-500 text-white text-xs font-bold rounded-full">
          {transactions.length}
        </span>
      )}
    </button>
  )
}

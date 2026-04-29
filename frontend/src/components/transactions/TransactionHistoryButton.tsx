'use client'

import React from 'react'
import { Clock } from 'lucide-react'
import { useTransactionStatus } from '@/contexts/TransactionStatusProvider'

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

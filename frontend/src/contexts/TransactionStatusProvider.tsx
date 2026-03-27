'use client'

import React, { createContext, useContext, useState, useCallback, useEffect, ReactNode } from 'react'
import { toast } from 'sonner'

// Transaction status types
export type TransactionStatus = 'pending' | 'success' | 'failed' | 'open'

// Transaction data interface
export interface Transaction {
  id: string
  hash: string
  status: TransactionStatus
  type: string
  description: string
  timestamp: number
  network: 'testnet' | 'public'
  url?: string
}

// Provider props interface
interface TransactionStatusProviderProps {
  children: ReactNode
}

// Context interface
interface TransactionStatusContextValue {
  transactions: Transaction[]
  pendingCount: number
  addTransaction: (tx: Omit<Transaction, 'id' | 'timestamp'>) => string
  updateTransaction: (id: string, updates: Partial<Transaction>) => void
  getTransaction: (id: string) => Transaction | undefined
  clearHistory: () => void
  isDrawerOpen: boolean
  setDrawerOpen: (open: boolean) => void
}

// Create context with default values
const TransactionStatusContext = createContext<TransactionStatusContextValue | undefined>(undefined)

// StellarExpert base URLs
const STELLAR_EXPERT_URLS = {
  testnet: 'https://stellar.expert/xlm-testnet/tx/',
  public: 'https://stellar.expert/xlm-public/tx/',
}

// Maximum transactions to store
const MAX_TRANSACTIONS = 10

// Generate unique ID
const generateId = () => `tx_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`

// Format timestamp for display
const formatTimestamp = (timestamp: number): string => {
  const date = new Date(timestamp)
  return date.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function TransactionStatusProvider({ children }: TransactionStatusProviderProps) {
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [isDrawerOpen, setDrawerOpen] = useState(false)

  // Calculate pending count
  const pendingCount = transactions.filter(tx => tx.status === 'pending' || tx.status === 'open').length

  // Add a new transaction
  const addTransaction = useCallback((txData: Omit<Transaction, 'id' | 'timestamp'>): string => {
    const id = generateId()
    const url = txData.hash 
      ? `${STELLAR_EXPERT_URLS[txData.network]}${txData.hash}`
      : undefined
    
    const newTransaction: Transaction = {
      ...txData,
      id,
      timestamp: Date.now(),
      url,
    }

    setTransactions(prev => {
      const updated = [newTransaction, ...prev]
      // Keep only last MAX_TRANSACTIONS
      return updated.slice(0, MAX_TRANSACTIONS)
    })

    // Show pending toast immediately
    toast.loading(`Submitting transaction: ${txData.description}`, {
      id,
      duration: Infinity, // Keep until updated
    })

    return id
  }, [])

  // Update an existing transaction
  const updateTransaction = useCallback((id: string, updates: Partial<Transaction>) => {
    setTransactions(prev => {
      const txIndex = prev.findIndex(tx => tx.id === id)
      if (txIndex === -1) return prev

      const updatedTx = { ...prev[txIndex], ...updates }
      const updated = [...prev]
      updated[txIndex] = updatedTx

      // Show toast based on new status
      const toastMessage = updates.status === 'success'
        ? `Transaction successful: ${updatedTx.description}`
        : updates.status === 'failed'
        ? `Transaction failed: ${updatedTx.description}`
        : null

      if (toastMessage) {
        if (updates.status === 'success') {
          toast.success(
            <div className="flex flex-col gap-1">
              <span className="font-medium">{toastMessage}</span>
              {updatedTx.url && (
                <a 
                  href={updatedTx.url} 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="text-xs text-blue-600 hover:underline"
                  onClick={(e) => e.stopPropagation()}
                >
                  View on StellarExpert
                </a>
              )}
            </div>,
            { id, duration: 5000 }
          )
        } else if (updates.status === 'failed') {
          toast.error(
            <div className="flex flex-col gap-1">
              <span className="font-medium">{toastMessage}</span>
              {updates.url && (
                <a 
                  href={updates.url} 
                  target="_blank" 
                  rel="noopener noreferrer"
                  className="text-xs text-blue-600 hover:underline"
                  onClick={(e) => e.stopPropagation()}
                >
                  View on StellarExpert
                </a>
              )}
            </div>,
            { id, duration: 8000 }
          )
        }
      }

      return updated
    })
  }, [])

  // Get a specific transaction
  const getTransaction = useCallback((id: string): Transaction | undefined => {
    return transactions.find(tx => tx.id === id)
  }, [transactions])

  // Clear transaction history
  const clearHistory = useCallback(() => {
    setTransactions([])
  }, [])

  // Load transactions from localStorage on mount
  useEffect(() => {
    try {
      const stored = localStorage.getItem('stellovault_transactions')
      if (stored) {
        const parsed = JSON.parse(stored)
        // Only load transactions from the last 24 hours
        const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000
        const recentTxs = parsed.filter((tx: Transaction) => tx.timestamp > oneDayAgo)
        setTransactions(recentTxs)
      }
    } catch (error) {
      console.warn('Failed to load transactions from localStorage:', error)
    }
  }, [])

  // Persist transactions to localStorage
  useEffect(() => {
    try {
      localStorage.setItem('stellovault_transactions', JSON.stringify(transactions))
    } catch (error) {
      console.warn('Failed to save transactions to localStorage:', error)
    }
  }, [transactions])

  const value: TransactionStatusContextValue = {
    transactions,
    pendingCount,
    addTransaction,
    updateTransaction,
    getTransaction,
    clearHistory,
    isDrawerOpen,
    setDrawerOpen,
  }

  return (
    <TransactionStatusContext.Provider value={value}>
      {children}
    </TransactionStatusContext.Provider>
  )
}

// Custom hook to use the transaction context
export function useTransactionStatus() {
  const context = useContext(TransactionStatusContext)
  if (context === undefined) {
    throw new Error('useTransactionStatus must be used within a TransactionStatusProvider')
  }
  return context
}

// Helper function to submit a transaction and handle status updates
export async function submitTransaction(
  context: TransactionStatusContextValue,
  transactionBuilder: () => Promise<{ hash: string; source: string }>,
  type: string,
  description: string,
  network: 'testnet' | 'public' = 'testnet'
): Promise<{ hash: string; id: string }> {
  // Add pending transaction
  const id = context.addTransaction({
    hash: '',
    status: 'pending',
    type,
    description,
    network,
  })

  try {
    // Submit the transaction
    const result = await transactionBuilder()
    
    // Update with hash
    context.updateTransaction(id, {
      hash: result.hash,
      url: `${STELLAR_EXPERT_URLS[network]}${result.hash}`,
    })

    return { hash: result.hash, id }
  } catch (error) {
    // Update as failed
    context.updateTransaction(id, {
      status: 'failed',
    })
    throw error
  }
}

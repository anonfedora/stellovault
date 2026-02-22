// Stellar/Soroban utility functions for StelloVault

import { Address, Contract, Networks } from '@stellar/stellar-sdk';

// Network configurations
export const NETWORKS = {
  testnet: {
    networkPassphrase: Networks.TESTNET,
    soroban: 'https://soroban-testnet.stellar.org',
  },
  mainnet: {
    networkPassphrase: Networks.PUBLIC,
    soroban: 'https://soroban.stellar.org',
  },
};

// Contract IDs (would be deployed contract addresses)
export const CONTRACT_IDS = {
  testnet: {
    stellovault: 'CA7QYNF7SOWQ3GLR2BGMZEHXAVIRZA4KVWLTJJFC7MGXUA74P7UJVSGZ', // placeholder
  },
  mainnet: {
    stellovault: '', // to be filled after deployment
  },
};

// Helper function to format amounts (Stellar uses 7 decimal places)
export const formatAmount = (amount: number | string): string => {
  const numAmount = typeof amount === 'string' ? parseFloat(amount) : amount;
  return (numAmount / 10000000).toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 7,
  });
};

// Convert human-readable amount to Stellar format
export const toStellarAmount = (amount: number): string => {
  return (amount * 10000000).toString();
};

// Validate Stellar address
export const isValidAddress = (address: string): boolean => {
  try {
    new Address(address);
    return true;
  } catch {
    return false;
  }
};

// Shorten address for display
export const shortenAddress = (address: string, chars = 4): string => {
  if (!address) return '';
  return `${address.slice(0, chars + 2)}...${address.slice(-chars)}`;
};

// Get explorer URL for transaction
export const getExplorerUrl = (txHash: string, network: 'testnet' | 'mainnet' = 'testnet'): string => {
  const baseUrl = network === 'testnet'
    ? 'https://stellar.expert/explorer/testnet/tx/'
    : 'https://stellar.expert/explorer/public/tx/';
  return `${baseUrl}${txHash}`;
};

// Contract interaction helpers
export const createContractInstance = (contractId: string) => {
  return new Contract(contractId);
};

// Error handling for Soroban transactions
export const handleContractError = (error: Error | { message?: string } | null | undefined): string => {
  if (error && 'message' in error && error.message?.includes('insufficient balance')) {
    return 'Insufficient balance for this transaction';
  }
  if (error && 'message' in error && error.message?.includes('unauthorized')) {
    return 'You are not authorized to perform this action';
  }
  if (error && 'message' in error && error.message?.includes('invalid amount')) {
    return 'Invalid amount specified';
  }
  return (error && 'message' in error && error.message) || 'An unexpected error occurred';
};
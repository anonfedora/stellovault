'use client';

import { useState, useEffect } from 'react';

export type KycStatus = 'unverified' | 'pending' | 'verified' | 'rejected' | 'expired';

interface KycStatusResponse {
  user_id: string;
  kyc_status: KycStatus;
  kyc_expiry: string | null;
  is_valid: boolean;
  requires_verification: boolean;
}

interface UseKycStatusReturn {
  kycStatus: KycStatus | null;
  kycExpiry: string | null;
  isValid: boolean;
  requiresVerification: boolean;
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
  initiateVerification: () => Promise<void>;
}

export function useKycStatus(): UseKycStatusReturn {
  const [kycStatus, setKycStatus] = useState<KycStatus | null>(null);
  const [kycExpiry, setKycExpiry] = useState<string | null>(null);
  const [isValid, setIsValid] = useState(false);
  const [requiresVerification, setRequiresVerification] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchKycStatus = async () => {
    try {
      setIsLoading(true);
      setError(null);

      const response = await fetch('/api/kyc/status', {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Failed to fetch KYC status');
      }

      const data = await response.json();
      
      if (data.success && data.data) {
        const kycData: KycStatusResponse = data.data;
        setKycStatus(kycData.kyc_status);
        setKycExpiry(kycData.kyc_expiry);
        setIsValid(kycData.is_valid);
        setRequiresVerification(kycData.requires_verification);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error fetching KYC status:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const initiateVerification = async () => {
    try {
      setError(null);

      const response = await fetch('/api/kyc/initiate', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('access_token')}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error('Failed to initiate verification');
      }

      const data = await response.json();
      
      if (data.success && data.data) {
        setKycStatus(data.data.kyc_status);
        // In production, this would redirect to the KYC provider's verification flow
        alert('Verification initiated! In production, you would be redirected to the KYC provider.');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error initiating verification:', err);
      throw err;
    }
  };

  useEffect(() => {
    fetchKycStatus();
  }, []);

  return {
    kycStatus,
    kycExpiry,
    isValid,
    requiresVerification,
    isLoading,
    error,
    refetch: fetchKycStatus,
    initiateVerification,
  };
}

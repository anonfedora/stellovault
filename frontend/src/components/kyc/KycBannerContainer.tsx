'use client';

import { useKycStatus } from '@/hooks/useKycStatus';
import { KycVerificationBanner } from './KycVerificationBanner';

/**
 * Container component that fetches KYC status and displays the banner
 * Add this to your dashboard or main layout to show KYC warnings
 */
export function KycBannerContainer() {
  const {
    kycStatus,
    kycExpiry,
    requiresVerification,
    isLoading,
    initiateVerification,
  } = useKycStatus();

  // Don't show anything while loading
  if (isLoading) {
    return null;
  }

  // Don't show if no KYC status or doesn't require verification
  if (!kycStatus || !requiresVerification) {
    return null;
  }

  return (
    <KycVerificationBanner
      kycStatus={kycStatus}
      kycExpiry={kycExpiry}
      onInitiateVerification={initiateVerification}
    />
  );
}

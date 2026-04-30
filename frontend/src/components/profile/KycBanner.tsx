'use client';

import { Shield, AlertCircle, CheckCircle, Clock, XCircle, ArrowRight } from 'lucide-react';
import { KycStatus } from '@/hooks/useProfile';

interface KycBannerProps {
  status: KycStatus;
  onInitiateVerification: () => void;
}

export function KycBanner({ status, onInitiateVerification }: KycBannerProps) {
  const getBannerConfig = (kycStatus: KycStatus) => {
    switch (kycStatus) {
      case 'unverified':
        return {
          bgColor: 'bg-blue-50',
          borderColor: 'border-blue-200',
          icon: AlertCircle,
          iconColor: 'text-blue-600',
          title: 'Complete KYC Verification',
          description: 'Verify your identity to unlock full platform features and higher transaction limits.',
          buttonText: 'Start Verification',
          buttonColor: 'bg-blue-600 hover:bg-blue-700',
        };
      case 'pending':
        return {
          bgColor: 'bg-yellow-50',
          borderColor: 'border-yellow-200',
          icon: Clock,
          iconColor: 'text-yellow-600',
          title: 'Verification in Progress',
          description: 'Your KYC verification is being reviewed. This typically takes 1-2 business days.',
          buttonText: 'Check Status',
          buttonColor: 'bg-yellow-600 hover:bg-yellow-700',
        };
      case 'rejected':
        return {
          bgColor: 'bg-red-50',
          borderColor: 'border-red-200',
          icon: XCircle,
          iconColor: 'text-red-600',
          title: 'Verification Failed',
          description: 'Your KYC verification was rejected. Please review the feedback and try again.',
          buttonText: 'Retry Verification',
          buttonColor: 'bg-red-600 hover:bg-red-700',
        };
      case 'expired':
        return {
          bgColor: 'bg-orange-50',
          borderColor: 'border-orange-200',
          icon: AlertCircle,
          iconColor: 'text-orange-600',
          title: 'KYC Verification Expired',
          description: 'Your KYC verification has expired. Please re-verify to maintain full access.',
          buttonText: 'Re-verify',
          buttonColor: 'bg-orange-600 hover:bg-orange-700',
        };
      default:
        return null;
    }
  };

  const config = getBannerConfig(status);
  if (!config) return null;

  const Icon = config.icon;

  return (
    <div className={`${config.bgColor} border ${config.borderColor} rounded-lg p-6 mb-6`}>
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-4">
          <div className={`p-2 rounded-lg ${config.iconColor} bg-white`}>
            <Icon className="w-6 h-6" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-gray-900 mb-1">{config.title}</h3>
            <p className="text-gray-700">{config.description}</p>
          </div>
        </div>

        <button
          onClick={onInitiateVerification}
          className={`flex items-center space-x-2 px-4 py-2 text-white font-medium rounded-lg transition-colors ${config.buttonColor}`}
        >
          <span>{config.buttonText}</span>
          <ArrowRight className="w-4 h-4" />
        </button>
      </div>

      {status === 'unverified' && (
        <div className="mt-4 pt-4 border-t border-gray-200">
          <p className="text-sm text-gray-600 mb-2">Benefits of verification:</p>
          <ul className="text-sm text-gray-600 space-y-1">
            <li className="flex items-center space-x-2">
              <CheckCircle className="w-4 h-4 text-green-600" />
              <span>Higher transaction limits</span>
            </li>
            <li className="flex items-center space-x-2">
              <CheckCircle className="w-4 h-4 text-green-600" />
              <span>Access to all trading features</span>
            </li>
            <li className="flex items-center space-x-2">
              <CheckCircle className="w-4 h-4 text-green-600" />
              <span>Priority customer support</span>
            </li>
            <li className="flex items-center space-x-2">
              <CheckCircle className="w-4 h-4 text-green-600" />
              <span>Reduced fees on large transactions</span>
            </li>
          </ul>
        </div>
      )}
    </div>
  );
}

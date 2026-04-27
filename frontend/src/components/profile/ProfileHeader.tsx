'use client';

import { User, Shield, Calendar, Clock } from 'lucide-react';
import { KycStatus } from '@/hooks/useProfile';

interface ProfileHeaderProps {
  name?: string;
  stellarAddress: string;
  role: string;
  kycStatus: KycStatus;
  kycExpiry?: string;
  createdAt: string;
  onEdit?: () => void;
}

export function ProfileHeader({
  name,
  stellarAddress,
  role,
  kycStatus,
  kycExpiry,
  createdAt,
  onEdit,
}: ProfileHeaderProps) {
  const getKycStatusColor = (status: KycStatus) => {
    switch (status) {
      case 'verified':
        return 'bg-green-100 text-green-800 border-green-200';
      case 'pending':
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case 'rejected':
        return 'bg-red-100 text-red-800 border-red-200';
      case 'expired':
        return 'bg-orange-100 text-orange-800 border-orange-200';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  const getKycStatusText = (status: KycStatus) => {
    switch (status) {
      case 'verified':
        return 'Verified';
      case 'pending':
        return 'Pending';
      case 'rejected':
        return 'Rejected';
      case 'expired':
        return 'Expired';
      default:
        return 'Unverified';
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  const truncatedAddress = `${stellarAddress.slice(0, 8)}...${stellarAddress.slice(-8)}`;

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
      <div className="flex items-start justify-between">
        <div className="flex items-start space-x-4">
          <div className="w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
            <User className="w-10 h-10 text-white" />
          </div>
          
          <div className="flex-1">
            <div className="flex items-center space-x-3 mb-2">
              <h1 className="text-2xl font-bold text-gray-900">
                {name || 'User'}
              </h1>
              <span className="px-3 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800 border border-blue-200">
                {role}
              </span>
            </div>
            
            <div className="flex items-center space-x-2 text-sm text-gray-600 mb-3">
              <code className="bg-gray-100 px-2 py-1 rounded font-mono">
                {truncatedAddress}
              </code>
            </div>

            <div className="flex items-center space-x-4 text-sm text-gray-500">
              <div className="flex items-center space-x-1">
                <Calendar className="w-4 h-4" />
                <span>Joined {formatDate(createdAt)}</span>
              </div>
              {kycExpiry && kycStatus === 'verified' && (
                <div className="flex items-center space-x-1">
                  <Clock className="w-4 h-4" />
                  <span>KYC expires {formatDate(kycExpiry)}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="flex flex-col items-end space-y-3">
          <div className={`px-4 py-2 rounded-lg border flex items-center space-x-2 ${getKycStatusColor(kycStatus)}`}>
            <Shield className="w-4 h-4" />
            <span className="font-medium">{getKycStatusText(kycStatus)}</span>
          </div>
          
          {onEdit && (
            <button
              onClick={onEdit}
              className="px-4 py-2 text-sm font-medium text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded-lg transition-colors"
            >
              Edit Profile
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

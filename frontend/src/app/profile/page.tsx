'use client';

import { useProfile } from '@/hooks/useProfile';
import { ProfileHeader } from '@/components/profile/ProfileHeader';
import { WalletManager } from '@/components/profile/WalletManager';
import { NotificationSettings } from '@/components/profile/NotificationSettings';
import { SecuritySettings } from '@/components/profile/SecuritySettings';
import { ActivityLog } from '@/components/profile/ActivityLog';
import { KycBanner } from '@/components/profile/KycBanner';
import { BarChart3, TrendingUp, Shield, Clock } from 'lucide-react';
import { toast } from 'sonner';

export default function ProfilePage() {
  const { profile, isLoading, error, refetch } = useProfile();

  if (isLoading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/4 mb-4" />
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-8" />
          <div className="h-32 bg-gray-200 rounded mb-6" />
        </div>
      </div>
    );
  }

  if (error || !profile) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <p className="text-red-800">Failed to load profile. Please try again.</p>
          <button
            onClick={() => refetch()}
            className="mt-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  const handleLogoutAllSessions = async () => {
    try {
      await fetch('/api/auth/logout-all', { method: 'POST', credentials: 'include' });
      window.location.href = '/login';
    } catch (error) {
      toast.error('Failed to logout from all sessions');
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">Profile</h1>
        <p className="text-gray-600 mb-8">
          Manage your account settings, wallet connections, and trading preferences.
        </p>

        {/* KYC Banner */}
        {profile.kycStatus !== 'verified' && (
          <KycBanner
            status={profile.kycStatus}
            onInitiateVerification={() => {
              toast.info('KYC verification flow would be initiated here');
            }}
          />
        )}

        {/* Profile Header */}
        <ProfileHeader
          name={profile.name}
          stellarAddress={profile.stellarAddress}
          role={profile.role}
          kycStatus={profile.kycStatus}
          kycExpiry={profile.kycExpiry}
          createdAt={profile.createdAt}
          onEdit={() => {
            toast.info('Profile edit modal would open here');
          }}
        />

        {/* Stats Overview */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <StatCard
            icon={BarChart3}
            label="Total Transactions"
            value="0"
            change="+0%"
          />
          <StatCard
            icon={TrendingUp}
            label="Active Escrows"
            value="0"
            change="+0%"
          />
          <StatCard
            icon={Shield}
            label="Security Score"
            value="85"
            change="+5%"
          />
          <StatCard
            icon={Clock}
            label="Account Age"
            value={getAccountAge(profile.createdAt)}
            change=""
          />
        </div>

        {/* Wallet Manager */}
        <WalletManager
          wallets={profile.wallets}
          onAddWallet={profile.addWallet}
          onRemoveWallet={profile.removeWallet}
          onSetPrimary={profile.setPrimaryWallet}
        />

        {/* Notification Settings */}
        <div className="mt-6">
          <NotificationSettings
            preferences={profile.notificationPreferences}
            onUpdate={profile.updateNotificationPreferences}
          />
        </div>

        {/* Security Settings */}
        <div className="mt-6">
          <SecuritySettings
            settings={profile.securitySettings}
            onUpdate={profile.updateSecuritySettings}
            onLogoutAllSessions={handleLogoutAllSessions}
          />
        </div>

        {/* Activity Log */}
        <div className="mt-6">
          <ActivityLog onFetch={profile.getActivityLog} />
        </div>
      </div>
    </div>
  );
}

function StatCard({
  icon: Icon,
  label,
  value,
  change,
}: {
  icon: any;
  label: string;
  value: string;
  change: string;
}) {
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
      <div className="flex items-center justify-between mb-2">
        <Icon className="w-5 h-5 text-gray-600" />
        {change && (
          <span className={`text-xs font-medium ${change.startsWith('+') ? 'text-green-600' : 'text-red-600'}`}>
            {change}
          </span>
        )}
      </div>
      <p className="text-2xl font-bold text-gray-900">{value}</p>
      <p className="text-sm text-gray-500">{label}</p>
    </div>
  );
}

function getAccountAge(createdAt: string): string {
  const created = new Date(createdAt);
  const now = new Date();
  const diffTime = Math.abs(now.getTime() - created.getTime());
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  if (diffDays < 30) {
    return `${diffDays} days`;
  } else if (diffDays < 365) {
    return `${Math.floor(diffDays / 30)} months`;
  } else {
    return `${Math.floor(diffDays / 365)} years`;
  }
}
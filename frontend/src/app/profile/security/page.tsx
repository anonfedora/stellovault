'use client';

import { useProfile } from '@/hooks/useProfile';
import { SecuritySettings } from '@/components/profile/SecuritySettings';
import { Shield, Lock, Key, Smartphone, AlertTriangle } from 'lucide-react';
import { toast } from 'sonner';

export default function SecurityPage() {
  const { profile, isLoading, updateSecuritySettings } = useProfile();

  const handleLogoutAllSessions = async () => {
    try {
      await fetch('/api/auth/logout-all', { method: 'POST', credentials: 'include' });
      toast.success('Logged out from all devices');
      window.location.href = '/login';
    } catch (error) {
      toast.error('Failed to logout from all devices');
    }
  };

  if (isLoading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="animate-pulse">
          <div className="h-8 bg-gray-200 rounded w-1/4 mb-4" />
          <div className="h-4 bg-gray-200 rounded w-1/2 mb-8" />
          <div className="h-32 bg-gray-200 rounded" />
        </div>
      </div>
    );
  }

  if (!profile) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <p className="text-red-800">Failed to load profile. Please try again.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">Security Settings</h1>
          <p className="text-gray-600">
            Manage your account security, authentication, and session preferences
          </p>
        </div>

        {/* Security Overview */}
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
          <div className="flex items-center space-x-2 mb-6">
            <Shield className="w-5 h-5 text-gray-600" />
            <h2 className="text-lg font-semibold text-gray-900">Security Overview</h2>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <SecurityCard
              icon={Lock}
              label="Password"
              status="Strong"
              statusColor="text-green-600"
              description="Last changed 30 days ago"
            />
            <SecurityCard
              icon={Key}
              label="2FA"
              status={profile.securitySettings.twoFactorEnabled ? 'Enabled' : 'Disabled'}
              statusColor={profile.securitySettings.twoFactorEnabled ? 'text-green-600' : 'text-yellow-600'}
              description={profile.securitySettings.twoFactorMethod === 'sms' ? 'SMS' : profile.securitySettings.twoFactorMethod === 'authenticator' ? 'Authenticator App' : 'Not configured'}
            />
            <SecurityCard
              icon={Smartphone}
              label="Active Sessions"
              status="1"
              statusColor="text-blue-600"
              description="Current device only"
            />
          </div>
        </div>

        {/* Security Settings Component */}
        <SecuritySettings
          settings={profile.securitySettings}
          onUpdate={updateSecuritySettings}
          onLogoutAllSessions={handleLogoutAllSessions}
        />

        {/* Security Tips */}
        <div className="mt-6 bg-blue-50 border border-blue-200 rounded-lg p-6">
          <div className="flex items-start space-x-3">
            <AlertTriangle className="w-5 h-5 text-blue-600 mt-0.5" />
            <div>
              <h3 className="font-semibold text-blue-900 mb-2">Security Best Practices</h3>
              <ul className="text-sm text-blue-800 space-y-1">
                <li>• Enable two-factor authentication for enhanced security</li>
                <li>• Use a strong, unique password for your account</li>
                <li>• Regularly review your active sessions and logout from unknown devices</li>
                <li>• Keep your software and browser up to date</li>
                <li>• Be cautious of phishing attempts and never share your credentials</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function SecurityCard({
  icon: Icon,
  label,
  status,
  statusColor,
  description,
}: {
  icon: any;
  label: string;
  status: string;
  statusColor: string;
  description: string;
}) {
  return (
    <div className="border border-gray-200 rounded-lg p-4">
      <div className="flex items-center justify-between mb-2">
        <Icon className="w-5 h-5 text-gray-600" />
        <span className={`text-sm font-medium ${statusColor}`}>{status}</span>
      </div>
      <p className="font-medium text-gray-900">{label}</p>
      <p className="text-sm text-gray-500">{description}</p>
    </div>
  );
}

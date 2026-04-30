'use client';

import { useState, useEffect, useCallback } from 'react';

export type KycStatus = 'unverified' | 'pending' | 'verified' | 'rejected' | 'expired';

export interface Wallet {
  id: string;
  address: string;
  isPrimary: boolean;
  label?: string;
  verifiedAt?: string;
}

export interface NotificationPreference {
  email: boolean;
  slack: boolean;
  sms: boolean;
  transactionAlerts: boolean;
  securityAlerts: boolean;
  marketingEmails: boolean;
}

export interface SecuritySettings {
  twoFactorEnabled: boolean;
  twoFactorMethod: 'sms' | 'authenticator' | null;
  sessionTimeout: number; // minutes
  ipWhitelist: string[];
  loginNotifications: boolean;
}

export interface ActivityLog {
  id: string;
  action: string;
  resourceId?: string;
  ipAddress?: string;
  timestamp: string;
  details?: Record<string, unknown>;
}

export interface UserProfile {
  id: string;
  stellarAddress: string;
  name?: string;
  role: 'BUYER' | 'SELLER' | 'LENDER' | 'AUDITOR' | 'ADMIN';
  kycStatus: KycStatus;
  kycExpiry?: string;
  createdAt: string;
  updatedAt: string;
  wallets: Wallet[];
  notificationPreferences: NotificationPreference;
  securitySettings: SecuritySettings;
}

interface UseProfileReturn {
  profile: UserProfile | null;
  isLoading: boolean;
  error: string | null;
  updateProfile: (data: Partial<UserProfile>) => Promise<void>;
  updateNotificationPreferences: (prefs: Partial<NotificationPreference>) => Promise<void>;
  updateSecuritySettings: (settings: Partial<SecuritySettings>) => Promise<void>;
  addWallet: (address: string, label?: string) => Promise<void>;
  removeWallet: (walletId: string) => Promise<void>;
  setPrimaryWallet: (walletId: string) => Promise<void>;
  getActivityLog: (limit?: number) => Promise<ActivityLog[]>;
  refetch: () => Promise<void>;
}

export function useProfile(): UseProfileReturn {
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchProfile = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);

      const response = await fetch('/api/users/profile', {
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to fetch profile');
      }

      const data = await response.json();
      setProfile(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error fetching profile:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const updateProfile = useCallback(async (data: Partial<UserProfile>) => {
    try {
      setError(null);

      const response = await fetch('/api/users/profile', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify(data),
      });

      if (!response.ok) {
        throw new Error('Failed to update profile');
      }

      const updatedProfile = await response.json();
      setProfile(updatedProfile);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error updating profile:', err);
      throw err;
    }
  }, []);

  const updateNotificationPreferences = useCallback(async (prefs: Partial<NotificationPreference>) => {
    try {
      setError(null);

      const response = await fetch('/api/users/notification-preferences', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify(prefs),
      });

      if (!response.ok) {
        throw new Error('Failed to update notification preferences');
      }

      const updatedPrefs = await response.json();
      
      setProfile((prev) => 
        prev ? { ...prev, notificationPreferences: { ...prev.notificationPreferences, ...updatedPrefs } } : null
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error updating notification preferences:', err);
      throw err;
    }
  }, []);

  const updateSecuritySettings = useCallback(async (settings: Partial<SecuritySettings>) => {
    try {
      setError(null);

      const response = await fetch('/api/users/security-settings', {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify(settings),
      });

      if (!response.ok) {
        throw new Error('Failed to update security settings');
      }

      const updatedSettings = await response.json();
      
      setProfile((prev) => 
        prev ? { ...prev, securitySettings: { ...prev.securitySettings, ...updatedSettings } } : null
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error updating security settings:', err);
      throw err;
    }
  }, []);

  const addWallet = useCallback(async (address: string, label?: string) => {
    try {
      setError(null);

      const response = await fetch('/api/wallets', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify({ address, label }),
      });

      if (!response.ok) {
        throw new Error('Failed to add wallet');
      }

      const newWallet = await response.json();
      
      setProfile((prev) => 
        prev ? { ...prev, wallets: [...prev.wallets, newWallet] } : null
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error adding wallet:', err);
      throw err;
    }
  }, []);

  const removeWallet = useCallback(async (walletId: string) => {
    try {
      setError(null);

      const response = await fetch(`/api/wallets/${walletId}`, {
        method: 'DELETE',
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to remove wallet');
      }

      setProfile((prev) => 
        prev ? { ...prev, wallets: prev.wallets.filter((w) => w.id !== walletId) } : null
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error removing wallet:', err);
      throw err;
    }
  }, []);

  const setPrimaryWallet = useCallback(async (walletId: string) => {
    try {
      setError(null);

      const response = await fetch(`/api/wallets/${walletId}/primary`, {
        method: 'PATCH',
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to set primary wallet');
      }

      setProfile((prev) => 
        prev ? {
          ...prev,
          wallets: prev.wallets.map((w) => ({
            ...w,
            isPrimary: w.id === walletId,
          })),
        } : null
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error setting primary wallet:', err);
      throw err;
    }
  }, []);

  const getActivityLog = useCallback(async (limit: number = 50): Promise<ActivityLog[]> => {
    try {
      setError(null);

      const response = await fetch(`/api/users/activity-log?limit=${limit}`, {
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to fetch activity log');
      }

      return await response.json();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Error fetching activity log:', err);
      return [];
    }
  }, []);

  useEffect(() => {
    fetchProfile();
  }, [fetchProfile]);

  return {
    profile,
    isLoading,
    error,
    updateProfile,
    updateNotificationPreferences,
    updateSecuritySettings,
    addWallet,
    removeWallet,
    setPrimaryWallet,
    getActivityLog,
    refetch: fetchProfile,
  };
}

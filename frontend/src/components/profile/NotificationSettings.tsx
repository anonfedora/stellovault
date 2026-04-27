'use client';

import { Bell, Mail, MessageSquare, Smartphone, Shield, Megaphone } from 'lucide-react';
import { NotificationPreference } from '@/hooks/useProfile';
import { toast } from 'sonner';

interface NotificationSettingsProps {
  preferences: NotificationPreference;
  onUpdate: (prefs: Partial<NotificationPreference>) => Promise<void>;
}

export function NotificationSettings({ preferences, onUpdate }: NotificationSettingsProps) {
  const handleToggle = async (key: keyof NotificationPreference) => {
    try {
      await onUpdate({ [key]: !preferences[key] });
      toast.success('Notification preferences updated');
    } catch (error) {
      toast.error('Failed to update preferences');
    }
  };

  const settings = [
    {
      category: 'Notification Channels',
      icon: Bell,
      items: [
        {
          key: 'email' as keyof NotificationPreference,
          label: 'Email Notifications',
          description: 'Receive notifications via email',
          icon: Mail,
        },
        {
          key: 'slack' as keyof NotificationPreference,
          label: 'Slack Notifications',
          description: 'Receive notifications in Slack',
          icon: MessageSquare,
        },
        {
          key: 'sms' as keyof NotificationPreference,
          label: 'SMS Notifications',
          description: 'Receive notifications via SMS',
          icon: Smartphone,
        },
      ],
    },
    {
      category: 'Alert Types',
      icon: Shield,
      items: [
        {
          key: 'transactionAlerts' as keyof NotificationPreference,
          label: 'Transaction Alerts',
          description: 'Get notified about transaction status changes',
          icon: Bell,
        },
        {
          key: 'securityAlerts' as keyof NotificationPreference,
          label: 'Security Alerts',
          description: 'Get notified about security events',
          icon: Shield,
        },
      ],
    },
    {
      category: 'Marketing',
      icon: Megaphone,
      items: [
        {
          key: 'marketingEmails' as keyof NotificationPreference,
          label: 'Marketing Emails',
          description: 'Receive product updates and promotional content',
          icon: Mail,
        },
      ],
    },
  ];

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center space-x-2 mb-6">
        <Bell className="w-5 h-5 text-gray-600" />
        <h2 className="text-lg font-semibold text-gray-900">Notification Preferences</h2>
      </div>

      <div className="space-y-8">
        {settings.map((section) => (
          <div key={section.category}>
            <div className="flex items-center space-x-2 mb-4">
              <section.icon className="w-4 h-4 text-gray-500" />
              <h3 className="text-sm font-medium text-gray-700">{section.category}</h3>
            </div>
            
            <div className="space-y-3 pl-6">
              {section.items.map((item) => {
                const Icon = item.icon;
                const isEnabled = preferences[item.key];
                
                return (
                  <div
                    key={item.key}
                    className="flex items-center justify-between p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                  >
                    <div className="flex items-center space-x-4">
                      <div className={`p-2 rounded-lg ${isEnabled ? 'bg-blue-100 text-blue-600' : 'bg-gray-100 text-gray-400'}`}>
                        <Icon className="w-5 h-5" />
                      </div>
                      <div>
                        <p className="font-medium text-gray-900">{item.label}</p>
                        <p className="text-sm text-gray-500">{item.description}</p>
                      </div>
                    </div>
                    
                    <button
                      onClick={() => handleToggle(item.key)}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        isEnabled ? 'bg-blue-600' : 'bg-gray-300'
                      }`}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          isEnabled ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        ))}
      </div>

      <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <p className="text-sm text-blue-800">
          <strong>Note:</strong> Changes to notification preferences take effect immediately.
          Some notifications may require additional setup (e.g., Slack webhook configuration).
        </p>
      </div>
    </div>
  );
}

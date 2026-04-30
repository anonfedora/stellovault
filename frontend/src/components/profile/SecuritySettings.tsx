'use client';

import { useState } from 'react';
import { Shield, Smartphone, Key, Clock, Globe, LogOut, AlertTriangle } from 'lucide-react';
import { SecuritySettings as SecuritySettingsType } from '@/hooks/useProfile';
import { toast } from 'sonner';

interface SecuritySettingsProps {
  settings: SecuritySettingsType;
  onUpdate: (settings: Partial<SecuritySettingsType>) => Promise<void>;
  onLogoutAllSessions?: () => Promise<void>;
}

export function SecuritySettings({ settings, onUpdate, onLogoutAllSessions }: SecuritySettingsProps) {
  const [isEnabling2FA, setIsEnabling2FA] = useState(false);
  const [newIpWhitelist, setNewIpWhitelist] = useState('');

  const handleToggle2FA = async () => {
    if (settings.twoFactorEnabled) {
      try {
        await onUpdate({ twoFactorEnabled: false, twoFactorMethod: null });
        toast.success('2FA disabled');
      } catch (error) {
        toast.error('Failed to disable 2FA');
      }
    } else {
      setIsEnabling2FA(true);
    }
  };

  const handleEnable2FA = async (method: 'sms' | 'authenticator') => {
    try {
      await onUpdate({ twoFactorEnabled: true, twoFactorMethod: method });
      setIsEnabling2FA(false);
      toast.success(`2FA enabled via ${method === 'sms' ? 'SMS' : 'Authenticator App'}`);
    } catch (error) {
      toast.error('Failed to enable 2FA');
    }
  };

  const handleSessionTimeoutChange = async (timeout: number) => {
    try {
      await onUpdate({ sessionTimeout: timeout });
      toast.success('Session timeout updated');
    } catch (error) {
      toast.error('Failed to update session timeout');
    }
  };

  const handleAddIpWhitelist = async () => {
    if (!newIpWhitelist.trim()) return;

    try {
      const updatedWhitelist = [...settings.ipWhitelist, newIpWhitelist.trim()];
      await onUpdate({ ipWhitelist: updatedWhitelist });
      setNewIpWhitelist('');
      toast.success('IP added to whitelist');
    } catch (error) {
      toast.error('Failed to add IP to whitelist');
    }
  };

  const handleRemoveIpWhitelist = async (ip: string) => {
    try {
      const updatedWhitelist = settings.ipWhitelist.filter((item) => item !== ip);
      await onUpdate({ ipWhitelist: updatedWhitelist });
      toast.success('IP removed from whitelist');
    } catch (error) {
      toast.error('Failed to remove IP from whitelist');
    }
  };

  const handleToggleLoginNotifications = async () => {
    try {
      await onUpdate({ loginNotifications: !settings.loginNotifications });
      toast.success('Login notification settings updated');
    } catch (error) {
      toast.error('Failed to update login notifications');
    }
  };

  const handleLogoutAllSessions = async () => {
    if (!onLogoutAllSessions) return;
    
    if (confirm('Are you sure you want to log out from all devices?')) {
      try {
        await onLogoutAllSessions();
        toast.success('Logged out from all devices');
      } catch (error) {
        toast.error('Failed to log out from all devices');
      }
    }
  };

  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
      <div className="flex items-center space-x-2 mb-6">
        <Shield className="w-5 h-5 text-gray-600" />
        <h2 className="text-lg font-semibold text-gray-900">Security Settings</h2>
      </div>

      <div className="space-y-8">
        {/* Two-Factor Authentication */}
        <div>
          <div className="flex items-center space-x-2 mb-4">
            <Key className="w-4 h-4 text-gray-500" />
            <h3 className="text-sm font-medium text-gray-700">Two-Factor Authentication</h3>
          </div>

          {isEnabling2FA ? (
            <div className="pl-6 space-y-3">
              <p className="text-sm text-gray-600 mb-4">Choose your preferred 2FA method:</p>
              <div className="grid grid-cols-2 gap-4">
                <button
                  onClick={() => handleEnable2FA('sms')}
                  className="flex items-center space-x-3 p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <Smartphone className="w-5 h-5 text-gray-600" />
                  <div className="text-left">
                    <p className="font-medium text-gray-900">SMS</p>
                    <p className="text-xs text-gray-500">Receive codes via text message</p>
                  </div>
                </button>
                <button
                  onClick={() => handleEnable2FA('authenticator')}
                  className="flex items-center space-x-3 p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors"
                >
                  <Key className="w-5 h-5 text-gray-600" />
                  <div className="text-left">
                    <p className="font-medium text-gray-900">Authenticator App</p>
                    <p className="text-xs text-gray-500">Use Google Authenticator or similar</p>
                  </div>
                </button>
              </div>
              <button
                onClick={() => setIsEnabling2FA(false)}
                className="text-sm text-gray-600 hover:text-gray-800"
              >
                Cancel
              </button>
            </div>
          ) : (
            <div className="pl-6 p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium text-gray-900">
                    {settings.twoFactorEnabled ? '2FA Enabled' : '2FA Disabled'}
                  </p>
                  <p className="text-sm text-gray-500">
                    {settings.twoFactorEnabled
                      ? `Using ${settings.twoFactorMethod === 'sms' ? 'SMS' : 'Authenticator App'}`
                      : 'Add an extra layer of security to your account'}
                  </p>
                </div>
                <button
                  onClick={handleToggle2FA}
                  className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
                    settings.twoFactorEnabled
                      ? 'bg-red-100 text-red-700 hover:bg-red-200'
                      : 'bg-blue-600 text-white hover:bg-blue-700'
                  }`}
                >
                  {settings.twoFactorEnabled ? 'Disable' : 'Enable'}
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Session Timeout */}
        <div>
          <div className="flex items-center space-x-2 mb-4">
            <Clock className="w-4 h-4 text-gray-500" />
            <h3 className="text-sm font-medium text-gray-700">Session Timeout</h3>
          </div>

          <div className="pl-6">
            <div className="flex items-center space-x-4">
              <select
                value={settings.sessionTimeout}
                onChange={(e) => handleSessionTimeoutChange(parseInt(e.target.value))}
                className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                <option value={15}>15 minutes</option>
                <option value={30}>30 minutes</option>
                <option value={60}>1 hour</option>
                <option value={120}>2 hours</option>
                <option value={240}>4 hours</option>
                <option value={480}>8 hours</option>
              </select>
              <p className="text-sm text-gray-500">
                Auto-logout after inactivity
              </p>
            </div>
          </div>
        </div>

        {/* IP Whitelist */}
        <div>
          <div className="flex items-center space-x-2 mb-4">
            <Globe className="w-4 h-4 text-gray-500" />
            <h3 className="text-sm font-medium text-gray-700">IP Whitelist</h3>
          </div>

          <div className="pl-6 space-y-3">
            <div className="flex space-x-2">
              <input
                type="text"
                value={newIpWhitelist}
                onChange={(e) => setNewIpWhitelist(e.target.value)}
                placeholder="Add IP address or CIDR (e.g., 192.168.1.1 or 192.168.1.0/24)"
                className="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <button
                onClick={handleAddIpWhitelist}
                className="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 transition-colors"
              >
                Add
              </button>
            </div>

            {settings.ipWhitelist.length > 0 ? (
              <div className="space-y-2">
                {settings.ipWhitelist.map((ip) => (
                  <div
                    key={ip}
                    className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
                  >
                    <code className="text-sm text-gray-700">{ip}</code>
                    <button
                      onClick={() => handleRemoveIpWhitelist(ip)}
                      className="text-red-600 hover:text-red-700 text-sm"
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-sm text-gray-500">No IP addresses whitelisted</p>
            )}
          </div>
        </div>

        {/* Login Notifications */}
        <div>
          <div className="flex items-center space-x-2 mb-4">
            <AlertTriangle className="w-4 h-4 text-gray-500" />
            <h3 className="text-sm font-medium text-gray-700">Login Notifications</h3>
          </div>

          <div className="pl-6 p-4 border border-gray-200 rounded-lg">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-900">Alert on New Login</p>
                <p className="text-sm text-gray-500">
                  Get notified when someone logs into your account
                </p>
              </div>
              <button
                onClick={handleToggleLoginNotifications}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                  settings.loginNotifications ? 'bg-blue-600' : 'bg-gray-300'
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    settings.loginNotifications ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>
          </div>
        </div>

        {/* Session Management */}
        {onLogoutAllSessions && (
          <div>
            <div className="flex items-center space-x-2 mb-4">
              <LogOut className="w-4 h-4 text-gray-500" />
              <h3 className="text-sm font-medium text-gray-700">Session Management</h3>
            </div>

            <div className="pl-6">
              <button
                onClick={handleLogoutAllSessions}
                className="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-lg hover:bg-red-200 transition-colors"
              >
                Log Out from All Devices
              </button>
              <p className="text-sm text-gray-500 mt-2">
                This will sign you out from all devices, including this one
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

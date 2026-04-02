"use client";

import { useState } from "react";
import { AlertCircle, X, Clock, XCircle } from "lucide-react";

export type KycStatus =
  | "unverified"
  | "pending"
  | "verified"
  | "rejected"
  | "expired";

interface KycVerificationBannerProps {
  kycStatus: KycStatus;
  kycExpiry?: string | null;
  onInitiateVerification?: () => void;
  onDismiss?: () => void;
}

export function KycVerificationBanner({
  kycStatus,
  kycExpiry,
  onInitiateVerification,
  onDismiss,
}: KycVerificationBannerProps) {
  const [isDismissed, setIsDismissed] = useState(false);

  // Don't show banner if verified and not expired
  if (
    kycStatus === "verified" &&
    (!kycExpiry || new Date(kycExpiry) > new Date())
  ) {
    return null;
  }

  // Don't show if dismissed
  if (isDismissed) {
    return null;
  }

  const handleDismiss = () => {
    setIsDismissed(true);
    onDismiss?.();
  };

  const getBannerConfig = () => {
    switch (kycStatus) {
      case "unverified":
        return {
          icon: AlertCircle,
          bgColor: "bg-yellow-50 dark:bg-yellow-900/20",
          borderColor: "border-yellow-200 dark:border-yellow-800",
          iconColor: "text-yellow-600 dark:text-yellow-400",
          textColor: "text-yellow-900 dark:text-yellow-100",
          title: "Verification Required",
          message:
            "Complete identity verification to create escrows over $10,000.",
          actionText: "Start Verification",
          showAction: true,
        };
      case "pending":
        return {
          icon: Clock,
          bgColor: "bg-blue-50 dark:bg-blue-900/20",
          borderColor: "border-blue-200 dark:border-blue-800",
          iconColor: "text-blue-600 dark:text-blue-400",
          textColor: "text-blue-900 dark:text-blue-100",
          title: "Verification Pending",
          message:
            "Your identity verification is being processed. This usually takes 1-2 business days.",
          actionText: null,
          showAction: false,
        };
      case "rejected":
        return {
          icon: XCircle,
          bgColor: "bg-red-50 dark:bg-red-900/20",
          borderColor: "border-red-200 dark:border-red-800",
          iconColor: "text-red-600 dark:text-red-400",
          textColor: "text-red-900 dark:text-red-100",
          title: "Verification Rejected",
          message:
            "Your identity verification was not approved. Please contact support for assistance.",
          actionText: "Contact Support",
          showAction: true,
        };
      case "expired":
        return {
          icon: AlertCircle,
          bgColor: "bg-orange-50 dark:bg-orange-900/20",
          borderColor: "border-orange-200 dark:border-orange-800",
          iconColor: "text-orange-600 dark:text-orange-400",
          textColor: "text-orange-900 dark:text-orange-100",
          title: "Verification Expired",
          message:
            "Your identity verification has expired. Please verify again to continue.",
          actionText: "Renew Verification",
          showAction: true,
        };
      default:
        return null;
    }
  };

  const config = getBannerConfig();
  if (!config) return null;

  const Icon = config.icon;

  return (
    <div
      className={`${config.bgColor} ${config.borderColor} border rounded-lg p-4 mb-6 relative`}
      role="alert"
    >
      <div className="flex items-start gap-3">
        <Icon
          className={`${config.iconColor} shrink-0 mt-0.5`}
          size={20}
        />

        <div className="flex-1 min-w-0">
          <h3 className={`${config.textColor} font-semibold text-sm mb-1`}>
            {config.title}
          </h3>
          <p className={`${config.textColor} text-sm opacity-90`}>
            {config.message}
          </p>

          {config.showAction && config.actionText && (
            <button
              onClick={onInitiateVerification}
              className={`mt-3 px-4 py-2 text-sm font-medium rounded-md transition-colors
                ${
                  kycStatus === "rejected"
                    ? "bg-red-600 hover:bg-red-700 text-white"
                    : "bg-yellow-600 hover:bg-yellow-700 text-white"
                }`}
            >
              {config.actionText}
            </button>
          )}
        </div>

        <button
          onClick={handleDismiss}
          className={`${config.iconColor} hover:opacity-70 transition-opacity shrink-0`}
          aria-label="Dismiss"
        >
          <X size={18} />
        </button>
      </div>
    </div>
  );
}

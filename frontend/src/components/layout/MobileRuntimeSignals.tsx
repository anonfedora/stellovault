"use client";

import { useEffect, useState } from "react";
import { WifiOff, X } from "lucide-react";

function reportMobileSignal(type: string, payload: Record<string, unknown>) {
  const body = JSON.stringify({
    type,
    payload,
    path: window.location.pathname,
    userAgent: window.navigator.userAgent,
    timestamp: new Date().toISOString(),
  });

  if (navigator.sendBeacon) {
    navigator.sendBeacon("/api/v1/analytics/mobile", new Blob([body], { type: "application/json" }));
    return;
  }

  void fetch("/api/v1/analytics/mobile", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
    keepalive: true,
  });
}

export function MobileRuntimeSignals() {
  const [isOffline, setIsOffline] = useState(() =>
    typeof navigator === "undefined" ? false : !navigator.onLine,
  );
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    if ("serviceWorker" in navigator) {
      void navigator.serviceWorker.register("/sw.js");
    }

    const updateNetwork = () => {
      const offline = !navigator.onLine;
      setIsOffline(offline);
      reportMobileSignal(offline ? "offline" : "online", {});
    };

    const captureError = (event: ErrorEvent) => {
      reportMobileSignal("error", {
        message: event.message,
        source: event.filename,
        line: event.lineno,
      });
    };

    const captureRejection = (event: PromiseRejectionEvent) => {
      reportMobileSignal("unhandledrejection", {
        reason: String(event.reason),
      });
    };

    window.addEventListener("online", updateNetwork);
    window.addEventListener("offline", updateNetwork);
    window.addEventListener("error", captureError);
    window.addEventListener("unhandledrejection", captureRejection);

    return () => {
      window.removeEventListener("online", updateNetwork);
      window.removeEventListener("offline", updateNetwork);
      window.removeEventListener("error", captureError);
      window.removeEventListener("unhandledrejection", captureRejection);
    };
  }, []);

  if (!isOffline || dismissed) return null;

  return (
    <div className="fixed inset-x-3 bottom-20 z-50 flex min-h-12 items-center gap-3 rounded-lg bg-gray-950 px-4 py-3 text-sm font-medium text-white shadow-xl md:hidden">
      <WifiOff className="h-4 w-4 shrink-0" />
      <span className="flex-1">Offline mode</span>
      <button type="button" onClick={() => setDismissed(true)} className="rounded p-1">
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

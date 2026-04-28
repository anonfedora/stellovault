import type { Metadata, Viewport } from "next";
import { AppProviders } from "@/components/providers/AppProviders";
import { TransactionStatusProvider } from "@/contexts/TransactionStatusProvider";
import { LanguageProvider } from '@/components/i18n/LanguageProvider';
import { Toaster } from "sonner";
import { TransactionHistoryDrawer } from "@/components/transactions/TransactionHistoryDrawer";
import { MobileBottomNav } from "@/components/layout/MobileBottomNav";
import { MobileRuntimeSignals } from "@/components/layout/MobileRuntimeSignals";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    template: "%s | StelloVault",
    default: "StelloVault - Decentralized Lending Protocol",
  },
  description: "A decentralized lending protocol built on the Stellar network with on-chain governance and automated collateral management.",
  manifest: "/manifest.webmanifest",
  appleWebApp: {
    capable: true,
    statusBarStyle: "default",
    title: "StelloVault",
  },
  alternates: {
    languages: {
      en: "/",
      es: "/es",
      fr: "/fr",
      ar: "/ar",
      he: "/he",
      zh: "/zh",
      ja: "/ja",
    },
  },
};

export const viewport: Viewport = {
  themeColor: "#0f3b82",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <LanguageProvider initialLocale="en">
          <AppProviders>
            <TransactionStatusProvider>
              {children}
              <Toaster 
                position="top-right"
                expand={false}
                richColors
                closeButton
                duration={5000}
                visibleToasts={3}
              />
              <TransactionHistoryDrawer />
              <MobileBottomNav />
              <MobileRuntimeSignals />
            </TransactionStatusProvider>
          </AppProviders>
        </LanguageProvider>
      </body>
    </html>
  );
}
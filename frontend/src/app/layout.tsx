import type { Metadata, Viewport } from "next";
import { AppProviders } from "@/components/providers/AppProviders";
import { TransactionStatusProvider } from "@/contexts/TransactionStatusProvider";
import { LanguageProvider } from '@/components/i18n/LanguageProvider';
import { Toaster } from "sonner";
import { TransactionHistoryDrawer } from "@/components/transactions/TransactionHistoryDrawer";
import { AppProviders } from "@/components/providers/AppProviders";
import { PerformanceMonitor } from "@/components/PerformanceMonitor";
import { ServiceWorkerRegistration } from "@/components/ServiceWorkerRegistration";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

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
      <head>
        <link rel="preload" href="/favicon.ico" as="image" />
      </head>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <TransactionStatusProvider>
          <AppProviders>
            <PerformanceMonitor />
            <ServiceWorkerRegistration />
            {children}
          </AppProviders>
        </LanguageProvider>
      </body>
    </html>
  );
}
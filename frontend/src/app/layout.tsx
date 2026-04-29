import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { TransactionStatusProvider } from "@/contexts/TransactionStatusProvider";
import { AppProviders } from "@/components/providers/AppProviders";
import { PerformanceMonitor } from "@/components/PerformanceMonitor";
import { ServiceWorkerRegistration } from "@/components/ServiceWorkerRegistration";
import dynamic from "next/dynamic";

const TransactionHistoryDrawer = dynamic(
  () => import("@/components/transactions/TransactionHistoryDrawer").then(mod => mod.TransactionHistoryDrawer),
  { ssr: false }
);

const Toaster = dynamic(
  () => import("sonner").then(mod => mod.Toaster),
  { ssr: false }
);

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "StelloVault - Decentralized Lending Protocol",
  description: "A decentralized lending protocol built on the Stellar network with on-chain governance and automated collateral management.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
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
          <Toaster 
            position="top-right"
            expand={false}
            richColors
            closeButton
            duration={5000}
            visibleToasts={3}
          />
          <TransactionHistoryDrawer />
        </TransactionStatusProvider>
      </body>
    </html>
  );
}

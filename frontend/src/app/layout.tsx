import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "sonner";
import { TransactionStatusProvider } from "@/contexts/TransactionStatusProvider";
import { TransactionHistoryDrawer } from "@/components/transactions/TransactionHistoryDrawer";
import { AppProviders } from "@/components/providers/AppProviders";

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
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
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
        </TransactionStatusProvider>
        <AppProviders>{children}</AppProviders>
      </body>
    </html>
  );
}

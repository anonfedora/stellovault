import type { Metadata, Viewport } from "next";
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
  return children;
}
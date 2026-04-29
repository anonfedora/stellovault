"use client";

import { useRouter } from "next/navigation";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { TokenizationForm } from "@/components/collateral/TokenizationForm";
import { useCollateral } from "@/hooks/useCollateral";
import type { CollateralDocument } from "@/hooks/useCollateral";

// Demo escrow ID — in production derive from user's active escrow
const DEMO_ESCROW_ID = "demo-escrow-id";
const DEMO_USER_ID = "demo-user-id";

export default function NewCollateralPage() {
  const router = useRouter();
  const { tokenizeAsset, loading } = useCollateral();

  async function handleSubmit(data: {
    escrowId: string;
    assetData: { assetType: string; amount: number; description: string; issuer: string };
    documents: CollateralDocument[];
  }) {
    await tokenizeAsset({ userId: DEMO_USER_ID, ...data });
    router.push("/collateral");
  }

  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-2xl">
        <Link href="/collateral" className="inline-flex items-center gap-1 text-sm text-gray-500 hover:text-gray-800">
          <ArrowLeft className="h-4 w-4" /> Back to collateral
        </Link>

        <div className="mt-6">
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">Tokenize New Asset</h1>
          <p className="mt-2 text-gray-600">
            Convert a real-world asset into a Stellar collateral token with embedded metadata.
          </p>
          <p className="mt-3 text-sm text-gray-500">
            Supported collateral types include invoices, commodities, receivables, inventory, and real estate. Attach proof documents and confirm the details before minting.
          </p>
        </div>

        <div className="mt-8 rounded-xl border border-gray-200 bg-white p-6">
          <TokenizationForm
            escrowId={DEMO_ESCROW_ID}
            onSubmit={handleSubmit}
            loading={loading}
          />
        </div>
      </div>
    </main>
  );
}

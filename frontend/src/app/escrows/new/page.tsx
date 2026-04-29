import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { EscrowForm } from "@/components/escrows/EscrowForm";

export default function NewEscrowPage() {
  return (
    <main className="min-h-screen bg-gray-50 px-4 pb-24 pt-6 text-gray-950 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-3xl">
        <Link href="/escrows" className="inline-flex items-center gap-2 text-sm font-semibold text-blue-800">
          <ArrowLeft className="h-4 w-4" />
          Back to escrows
        </Link>
        <div className="mb-6 mt-5">
          <p className="text-sm font-semibold text-blue-800">New escrow</p>
          <h1 className="mt-2 text-3xl font-bold">Create trade escrow</h1>
          <p className="mt-2 text-gray-600">
            Configure parties, funding terms, oracle checks, and release conditions.
          </p>
        </div>
        <EscrowForm />
      </div>
    </main>
  );
}

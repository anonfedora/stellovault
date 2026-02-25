"use client";

import Link from "next/link";
import { LoanForm } from "@/components/loans/LoanForm";
import { ArrowLeft } from "lucide-react";

export default function NewLoanPage() {
  const handleSubmit = async () => {
    // TODO: POST to /api/v1/loans with form data, handle response
    console.log("Loan submitted successfully");
  };

  return (
    <div className="container mx-auto px-4 py-8">
      {/* Back */}
      <Link
        href="/loans"
        className="inline-flex items-center gap-2 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 mb-6 transition text-sm"
      >
        <ArrowLeft className="w-4 h-4" /> Back to Loans
      </Link>

      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
          New Loan
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Tokenize collateral, define terms, and sign the transaction to issue a
          new loan.
        </p>
      </div>

      <LoanForm onSubmit={handleSubmit} />
    </div>
  );
}

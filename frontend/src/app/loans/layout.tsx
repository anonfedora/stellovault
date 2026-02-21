import { Sidebar } from "@/components/dashboard/Sidebar";

export default function LoansLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-screen bg-gray-50 dark:bg-black">
      <Sidebar />
      <main className="flex-1 ml-64 overflow-y-auto">{children}</main>
    </div>
  );
}

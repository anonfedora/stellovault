
import { Sidebar } from '@/components/dashboard/Sidebar';

export default function DashboardLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <div className="min-h-screen bg-gray-50 dark:bg-black lg:flex">
            <Sidebar />
            <main className="min-h-screen flex-1 overflow-y-auto pb-24 lg:ml-64 lg:pb-0">
                {children}
            </main>
        </div>
    );
}

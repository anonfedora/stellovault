"use client";

import Link from "next/link";
import { BriefcaseBusiness, Home, Landmark, LayoutDashboard, ShieldCheck } from "lucide-react";
import { usePathname } from "next/navigation";

const navItems = [
  { href: "/", label: "Home", icon: Home },
  { href: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { href: "/escrows", label: "Escrows", icon: ShieldCheck },
  { href: "/loans", label: "Loans", icon: BriefcaseBusiness },
  { href: "/governance", label: "Vote", icon: Landmark },
];

export function MobileBottomNav() {
  const pathname = usePathname();

  return (
    <nav className="fixed inset-x-0 bottom-0 z-40 border-t border-gray-200 bg-white/95 px-2 pb-[env(safe-area-inset-bottom)] pt-1 shadow-[0_-8px_24px_rgba(15,23,42,0.08)] backdrop-blur md:hidden">
      <div className="grid grid-cols-5">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = item.href === "/" ? pathname === "/" : pathname.startsWith(item.href);

          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex min-h-14 flex-col items-center justify-center gap-1 rounded-lg text-xs font-semibold ${
                isActive ? "text-blue-900" : "text-gray-500"
              }`}
            >
              <Icon className="h-5 w-5" />
              <span>{item.label}</span>
            </Link>
          );
        })}
      </div>
    </nav>
  );
}

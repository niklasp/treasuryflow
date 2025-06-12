"use client";

import type React from "react";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  CalendarDays,
  CreditCard,
  Menu,
  Search,
  User,
  Wallet,
  BarChart4,
  Plus,
  HandCoins,
  Vault,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Suspense } from "react";
import { cn } from "@/lib/utils";
import { AppNavbar } from "@/components/app-navbar";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const pathname = usePathname();

  const navigation = [
    {
      name: "Dashboard",
      href: "/dashboard",
      icon: BarChart4,
      color: "text-purple-500",
    },
    {
      name: "Payouts",
      href: "/payouts",
      icon: CreditCard,
      color: "text-rose-500",
    },
    // {
    //   name: "Schedule",
    //   href: "/",
    //   icon: CalendarDays,
    //   color: "text-blue-500",
    // },
  ];

  const bottomNavigation = [
    {
      name: "Add Payout",
      href: "/payouts/add",
      icon: HandCoins,
      color: "text-green-500",
      buttonVariant: "outline",
    },
    {
      name: "Create Treasury",
      href: "/create-treasury",
      icon: Vault,
      color: "text-green-500",
      buttonVariant: "outline",
    },
  ];

  return (
    <div className="flex min-h-screen flex-col">
      <AppNavbar
        isSidebarOpen={isSidebarOpen}
        setIsSidebarOpen={setIsSidebarOpen}
      />
      <div className="flex flex-1">
        <aside
          className={`${
            isSidebarOpen ? "flex" : "hidden"
          } w-64 flex-col border-r border-white/5 bg-black/20 backdrop-blur-md md:flex`}
        >
          <nav className="grid gap-2 p-4 text-sm">
            {navigation.map((item) => {
              const isActive = pathname === item.href;
              return (
                <Link
                  key={item.name}
                  href={item.href}
                  className={cn(
                    "flex items-center gap-3 rounded-lg px-3 py-2 transition-colors hover:text-foreground hover:bg-white/5",
                    isActive
                      ? "bg-gradient-to-r from-purple-900/20 to-indigo-900/20 text-primary-foreground"
                      : "text-muted-foreground"
                  )}
                >
                  <item.icon className={cn("h-4 w-4", item.color)} />
                  {item.name}
                </Link>
              );
            })}
            <div className="h-px w-full bg-white/5" />
            {bottomNavigation.map((item) => {
              const isActive = pathname === item.href;
              return (
                <Link
                  key={item.name}
                  href={item.href}
                  className={cn(
                    "flex items-center gap-3 rounded-lg px-3 py-2 transition-colors hover:text-foreground hover:bg-white/5",
                    isActive
                      ? "bg-gradient-to-r from-purple-900/20 to-indigo-900/20 text-primary-foreground"
                      : "text-muted-foreground",
                    item.buttonVariant === "outline" ? "bg-white/5" : ""
                  )}
                >
                  <Plus className="h-4 w-4" />
                  <item.icon className={cn("h-4 w-4", item.color)} />
                  {item.name}
                </Link>
              );
            })}
          </nav>
        </aside>
        <main className="flex-1 overflow-auto">
          <Suspense>{children}</Suspense>
        </main>
      </div>
    </div>
  );
}

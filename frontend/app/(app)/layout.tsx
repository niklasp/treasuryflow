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
      <header className="sticky top-0 z-10 flex h-16 items-center gap-4 border-b border-white/5 bg-background/80 backdrop-blur-md px-4 md:px-6">
        <Button
          variant="outline"
          size="icon"
          className="md:hidden"
          onClick={() => setIsSidebarOpen(!isSidebarOpen)}
        >
          <Menu className="h-5 w-5" />
          <span className="sr-only">Toggle sidebar</span>
        </Button>
        <div className="flex items-center gap-2">
          <span className="text-2xl">🪼</span>
          <span className="text-xl font-semibold tracking-tight">
            TreasuryFlow
          </span>
        </div>
        <div className="ml-auto flex items-center gap-4">
          <form className="hidden md:flex">
            <div className="relative">
              <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                type="search"
                placeholder="Search..."
                className="w-64 rounded-lg border-white/5 bg-black/20 pl-8 md:w-80 focus-visible:ring-primary"
              />
            </div>
          </form>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                className="rounded-full border-white/5 bg-black/20"
              >
                <User className="h-5 w-5 text-purple-500" />
                <span className="sr-only">Toggle user menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              className="border-white/5 bg-black/80 backdrop-blur-md"
            >
              <DropdownMenuLabel>My Account</DropdownMenuLabel>
              <DropdownMenuSeparator className="bg-white/5" />
              <DropdownMenuItem>Settings</DropdownMenuItem>
              <DropdownMenuItem>Support</DropdownMenuItem>
              <DropdownMenuSeparator className="bg-white/5" />
              <DropdownMenuItem>
                <Link href="/" className="flex w-full">
                  Logout
                </Link>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>
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
                      : "text-muted-foreground",
                    item.buttonVariant === "outline"
                      ? "border border-white/15 bg-black/20 hover:bg-black/40"
                      : ""
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

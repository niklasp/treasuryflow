import type React from "react";
import Link from "next/link";
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
import { Menu, Search, User } from "lucide-react";
import { WalletSelect } from "./account/wallet-select";

interface AppNavbarProps {
  isSidebarOpen: boolean;
  setIsSidebarOpen: (open: boolean) => void;
}

export function AppNavbar({ isSidebarOpen, setIsSidebarOpen }: AppNavbarProps) {
  return (
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
        <WalletSelect className="flex gap-2" />
      </div>
    </header>
  );
}

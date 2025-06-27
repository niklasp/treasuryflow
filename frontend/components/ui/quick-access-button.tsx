"use client";

import { IconCirclePlusFilled, IconWallet } from "@tabler/icons-react";
import { ChevronDown } from "lucide-react";
import Link from "next/link";
import { useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";
import { trimAddress } from "@/lib/utils";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SidebarMenuButton } from "@/components/ui/sidebar";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";

export function QuickAccessButton() {
  const { selectedAccount } = usePolkadotExtension();
  const treasuries = useQuery(
    api.treasuries.listByOwner,
    selectedAccount ? { owner: selectedAccount.address } : "skip"
  );

  // Loading state
  if (treasuries === undefined) {
    return (
      <SidebarMenuButton
        tooltip="Loading..."
        className="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground min-w-8 duration-200 ease-linear"
        disabled
      >
        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-primary-foreground"></div>
        <span>Add Payout</span>
      </SidebarMenuButton>
    );
  }

  // No treasuries - show "Add Treasury"
  if (treasuries.length === 0) {
    return (
      <SidebarMenuButton
        tooltip="Create your first treasury"
        className="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground min-w-8 duration-200 ease-linear"
        asChild
      >
        <Link href="/create-treasury">
          <IconWallet />
          <span>Add Treasury</span>
        </Link>
      </SidebarMenuButton>
    );
  }

  // Single treasury - direct link to add payout
  if (treasuries.length === 1) {
    return (
      <SidebarMenuButton
        tooltip="Add a new payout"
        className="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground min-w-8 duration-200 ease-linear"
        asChild
      >
        <Link href={`/new-payout/${treasuries[0].contractAddress}`}>
          <IconCirclePlusFilled />
          <span>Add Payout</span>
        </Link>
      </SidebarMenuButton>
    );
  }

  // Multiple treasuries - show dropdown
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <SidebarMenuButton
          tooltip="Add a new payout"
          className="bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground min-w-8 duration-200 ease-linear"
        >
          <IconCirclePlusFilled />
          <span>Add Payout</span>
          <ChevronDown className="h-4 w-4 ml-auto" />
        </SidebarMenuButton>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" side="right" className="w-80">
        <div className="px-2 py-1.5 text-sm font-semibold text-muted-foreground">
          Select Treasury
        </div>
        {treasuries.map((treasury) => (
          <DropdownMenuItem key={treasury._id} asChild>
            <Link
              href={`/new-payout/${treasury.contractAddress}`}
              className="flex items-start gap-3 p-3"
            >
              <IconWallet className="h-4 w-4 text-primary mt-0.5 flex-shrink-0" />
              <div className="flex flex-col gap-1 min-w-0 flex-1">
                <span className="font-medium truncate">{treasury.name}</span>
                {treasury.description && (
                  <span className="text-xs text-muted-foreground line-clamp-2">
                    {treasury.description}
                  </span>
                )}
                <span className="text-xs font-mono text-muted-foreground">
                  {trimAddress(treasury.contractAddress, 8)}
                </span>
              </div>
            </Link>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

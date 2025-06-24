"use client";

import {
  IconCreditCard,
  IconDotsVertical,
  IconLogout,
  IconNotification,
  IconUserCircle,
} from "@tabler/icons-react";

import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { Identicon } from "@polkadot/react-identicon";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { DialogView, MultiViewDialog } from "./ui/multi-view-dialog";
import { ViewSelectWallet } from "./account/view-select-wallet";
import { ViewSelectAccount } from "./account/view-select-account";

export function NavUser({ className }: { className?: string }) {
  const { selectedAccount, selectedExtensions } = usePolkadotExtension();

  const views: DialogView[] = [
    {
      title: `Connect Wallets (${selectedExtensions.length} connected)`,
      description:
        "Select a wallet to connect to your account. If you don't have a wallet installed, you can install one from the list.",
      content: ({ next, previous }) => (
        <ViewSelectWallet next={next} previous={previous} />
      ),
    },
    {
      title: "Select Account",
      description: "Select an account to use for app interactions",
      content: ({ previous }) => <ViewSelectAccount previous={previous} />,
    },
  ];

  const hasConnectedAccounts = selectedExtensions.some((extension) =>
    extension.getAccounts().some((account) => account.address)
  );

  return (
    <SidebarMenu>
      <SidebarMenuItem>
        <MultiViewDialog
          initialView={hasConnectedAccounts ? 1 : 0}
          trigger={
            <SidebarMenuButton
              size="lg"
              className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
            >
              <Avatar className="h-8 w-8 rounded-lg">
                {selectedAccount?.address && (
                  <Identicon
                    value={selectedAccount?.address}
                    size={32}
                    theme="polkadot"
                    className="[&>svg>circle:first-child]:fill-none aspect-square size-full"
                  />
                )}
                <AvatarFallback className="rounded-lg">U</AvatarFallback>
              </Avatar>
              <div className="grid flex-1 text-left text-sm leading-tight">
                <span className="truncate font-medium">
                  {selectedAccount?.name}
                </span>
                <span className="text-muted-foreground truncate text-xs">
                  {selectedAccount?.address}
                </span>
              </div>
              {/* <IconDotsVertical className="ml-auto size-4" /> */}
            </SidebarMenuButton>
          }
          views={views}
        />
      </SidebarMenuItem>
    </SidebarMenu>
  );
}

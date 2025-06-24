"use client";

import { Button } from "@/components/ui/button";
import Identicon from "@polkadot/react-identicon";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { MultiViewDialog, DialogView } from "@/components/ui/multi-view-dialog";
import { ViewSelectWallet } from "./view-select-wallet";
import { ViewSelectAccount } from "./view-select-account";
import { cn } from "@/lib/utils";

export function WalletSelect({
  className,
  placeholder,
}: {
  className?: string;
  placeholder?: string;
}) {
  const { selectedAccount, selectedExtensions } = usePolkadotExtension();

  const hasConnectedAccounts = selectedExtensions.some((extension) =>
    extension.getAccounts().some((account) => account.address)
  );

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

  return (
    <MultiViewDialog
      initialView={hasConnectedAccounts ? 1 : 0}
      trigger={
        <Button
          variant="outline"
          className={cn("transition-[min-width] duration-300", className)}
        >
          {selectedAccount?.name && (
            <span className="hidden sm:block max-w-[100px] truncate">
              {selectedAccount?.name}
            </span>
          )}
          {selectedAccount?.address && (
            <Identicon
              value={selectedAccount?.address}
              size={24}
              theme="polkadot"
              className="[&>svg>circle:first-child]:fill-none"
            />
          )}
        </Button>
      }
      views={views}
    />
  );
}

"use client";

import { HexString } from "polkadot-api";
import { useTreasuryContract } from "@/hooks/use-treasury-contract";
import { formatBalanceCompact } from "@/lib/format-balance";

interface TreasuryCardBalanceProps {
  contractAddress: HexString;
  ss58Address?: string;
}

export function TreasuryCardBalance({
  contractAddress,
}: TreasuryCardBalanceProps) {
  // Use SS58 address if available, otherwise validate contract address format
  const isValidAddress =
    contractAddress &&
    contractAddress.startsWith("0x") &&
    contractAddress.length >= 42;

  const { balance, isLoadingBalance, balanceError } = useTreasuryContract(
    isValidAddress
      ? contractAddress
      : ("0x0000000000000000000000000000000000000000" as HexString)
  );

  if (!isValidAddress) {
    return (
      <span className="text-xs text-muted-foreground">Invalid address</span>
    );
  }

  if (isLoadingBalance) {
    return (
      <div className="animate-pulse">
        <div className="h-4 bg-gray-300 rounded w-16"></div>
      </div>
    );
  }

  if (balanceError) {
    console.warn("Balance query error:", balanceError);
    return (
      <span className="text-xs text-muted-foreground">Unable to load</span>
    );
  }

  return (
    <span className="text-sm font-medium">
      {balance !== undefined ? formatBalanceCompact(balance) : "0 DOT"}
    </span>
  );
}

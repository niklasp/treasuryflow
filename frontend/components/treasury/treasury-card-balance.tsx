"use client";

import { HexString } from "polkadot-api";
import { useTreasuryContract } from "@/hooks/use-treasury-contract";
import { formatBalanceCompact } from "@/lib/format-balance";
import { NetworkId } from "@/lib/treasury-contract-service";

interface TreasuryCardBalanceProps {
  contractAddress: HexString;
  networkId: NetworkId;
  ss58Address?: string;
}

export function TreasuryCardBalance({
  contractAddress,
  networkId,
}: TreasuryCardBalanceProps) {
  // Validate contract address format
  const isValidAddress =
    contractAddress &&
    contractAddress.startsWith("0x") &&
    contractAddress.length >= 42 &&
    /^0x[0-9a-fA-F]+$/.test(contractAddress); // Ensure it's actually hex

  const { balance, isLoadingBalance, balanceError } = useTreasuryContract(
    isValidAddress
      ? contractAddress
      : ("0x0000000000000000000000000000000000000000" as HexString),
    networkId
  );

  if (!isValidAddress) {
    return (
      <span className="text-xs text-muted-foreground">Invalid address</span>
    );
  }

  if (isLoadingBalance) {
    return (
      <div className="animate-pulse">
        <div className="w-16 h-4 bg-gray-300 rounded"></div>
      </div>
    );
  }

  if (balanceError) {
    console.warn("Balance query error:", balanceError);
    const errorMessage =
      balanceError instanceof Error ? balanceError.message : "Unknown error";

    return (
      <span className="text-xs text-destructive" title={errorMessage}>
        Balance unavailable
      </span>
    );
  }

  return (
    <span className="text-sm font-medium">
      {balance !== undefined ? formatBalanceCompact(balance) : "0 DOT"}
    </span>
  );
}

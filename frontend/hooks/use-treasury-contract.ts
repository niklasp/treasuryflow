"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { HexString, FixedSizeBinary } from "polkadot-api";
import {
  createTreasuryContractService,
  NetworkId,
} from "@/lib/treasury-contract-service";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { useMemo } from "react";

// Query keys
const TREASURY_QUERY_KEYS = {
  balance: (contractAddress: HexString, networkId: NetworkId) =>
    ["treasury", "balance", contractAddress, networkId] as const,
  pendingPayouts: (
    contractAddress: HexString,
    accountAddress: string,
    networkId: NetworkId
  ) =>
    [
      "treasury",
      "pendingPayouts",
      contractAddress,
      accountAddress,
      networkId,
    ] as const,
  pendingPayoutIds: (
    contractAddress: HexString,
    accountAddress: string,
    networkId: NetworkId
  ) =>
    [
      "treasury",
      "pendingPayoutIds",
      contractAddress,
      accountAddress,
      networkId,
    ] as const,
};

export function useTreasuryContract(
  contractAddress: HexString,
  networkId: NetworkId = "POP_NETWORK", // Default to POP_NETWORK for backward compatibility
  ss58Address?: string
) {
  const { selectedAccount } = usePolkadotExtension();
  const queryClient = useQueryClient();

  // Create the service instance for the specific network
  const treasuryService = useMemo(
    () => createTreasuryContractService(networkId),
    [networkId]
  );

  // Check if contract address is valid
  const isValidContractAddress = Boolean(
    contractAddress &&
      contractAddress !== "0x0000000000000000000000000000000000000000" &&
      contractAddress.startsWith("0x") &&
      contractAddress.length >= 42 &&
      /^0x[0-9a-fA-F]+$/.test(contractAddress)
  );

  // Query for treasury balance using contract's get_balance function
  const balanceQuery = useQuery({
    queryKey: TREASURY_QUERY_KEYS.balance(contractAddress, networkId),
    queryFn: async () => {
      try {
        return await treasuryService.getBalance(contractAddress);
      } catch (error) {
        // Log the error for debugging
        console.warn("Balance query failed:", error);
        // Re-throw to let TanStack Query handle the error state
        throw error;
      }
    },
    enabled: isValidContractAddress,
    refetchInterval: 30000, // Refetch every 30 seconds only on success
    refetchIntervalInBackground: false, // Don't refetch when tab is not active
    retry: (failureCount, error) => {
      // Don't retry for certain types of errors
      if (error instanceof Error) {
        const errorMessage = error.message.toLowerCase();
        if (
          errorMessage.includes("checksum") ||
          errorMessage.includes("invalid") ||
          errorMessage.includes("contract not found") ||
          errorMessage.includes("does not exist")
        ) {
          return false; // Don't retry for invalid contract addresses
        }
      }
      // For other errors, retry up to 3 times
      return failureCount < 3;
    },
    retryDelay: (attemptIndex) => Math.min(30000, 1000 * 2 ** attemptIndex), // Exponential backoff capped at 30s
  });

  // Query for pending payouts
  const pendingPayoutsQuery = useQuery({
    queryKey: TREASURY_QUERY_KEYS.pendingPayouts(
      contractAddress,
      selectedAccount?.address || "",
      networkId
    ),
    queryFn: async () => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }
      return await treasuryService.getPendingPayouts(
        contractAddress,
        selectedAccount
      );
    },
    enabled: !!selectedAccount && !!contractAddress,
    refetchInterval: 30000, // Refetch every 30 seconds
  });

  // Query for pending payout IDs
  const pendingPayoutIdsQuery = useQuery({
    queryKey: TREASURY_QUERY_KEYS.pendingPayoutIds(
      contractAddress,
      selectedAccount?.address || "",
      networkId
    ),
    queryFn: async () => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }
      return await treasuryService.getPendingPayoutIds(
        contractAddress,
        selectedAccount
      );
    },
    enabled: !!selectedAccount && !!contractAddress,
    refetchInterval: 30000, // Refetch every 30 seconds
  });

  // Mutation for adding a single payout
  const addPayoutMutation = useMutation({
    mutationFn: async ({ to, amount }: { to: string; amount: bigint }) => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }
      return await treasuryService.addPayout(
        contractAddress,
        selectedAccount,
        to,
        amount
      );
    },
    onSuccess: () => {
      // Invalidate and refetch pending payouts data
      queryClient.invalidateQueries({
        queryKey: TREASURY_QUERY_KEYS.pendingPayouts(
          contractAddress,
          selectedAccount?.address || "",
          networkId
        ),
      });
      queryClient.invalidateQueries({
        queryKey: TREASURY_QUERY_KEYS.pendingPayoutIds(
          contractAddress,
          selectedAccount?.address || "",
          networkId
        ),
      });
    },
  });

  // Mutation for adding multiple payouts
  const addPayoutBatchMutation = useMutation({
    mutationFn: async ({
      payouts,
    }: {
      payouts: [FixedSizeBinary<20>, bigint][];
    }) => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }
      return await treasuryService.addPayoutBatch(
        contractAddress,
        selectedAccount,
        payouts
      );
    },
    onSuccess: () => {
      // Invalidate and refetch pending payouts data
      queryClient.invalidateQueries({
        queryKey: TREASURY_QUERY_KEYS.pendingPayouts(
          contractAddress,
          selectedAccount?.address || "",
          networkId
        ),
      });
      queryClient.invalidateQueries({
        queryKey: TREASURY_QUERY_KEYS.pendingPayoutIds(
          contractAddress,
          selectedAccount?.address || "",
          networkId
        ),
      });
    },
  });

  return {
    // Query data
    balance: balanceQuery.data,
    pendingPayouts: pendingPayoutsQuery.data,
    pendingPayoutIds: pendingPayoutIdsQuery.data,

    // Query states
    isLoadingBalance: balanceQuery.isLoading,
    isLoadingPayouts: pendingPayoutsQuery.isLoading,
    isLoadingPayoutIds: pendingPayoutIdsQuery.isLoading,
    balanceError: balanceQuery.error,
    payoutsError: pendingPayoutsQuery.error,
    payoutIdsError: pendingPayoutIdsQuery.error,

    // Mutations
    addPayout: addPayoutMutation.mutate,
    addPayoutBatch: addPayoutBatchMutation.mutate,

    // Mutation states
    isAddingPayout: addPayoutMutation.isPending,
    isAddingPayoutBatch: addPayoutBatchMutation.isPending,
    addPayoutError: addPayoutMutation.error,
    addPayoutBatchError: addPayoutBatchMutation.error,

    // Refetch functions
    refetchBalance: balanceQuery.refetch,
    refetchPayouts: pendingPayoutsQuery.refetch,
    refetchPayoutIds: pendingPayoutIdsQuery.refetch,
  };
}

// Separate hook for deployment (keeping the existing pattern)
export { useDeployTreasury } from "./use-deploy-treasury";

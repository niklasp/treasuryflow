"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { HexString, FixedSizeBinary } from "polkadot-api";
import { treasuryContractService } from "@/lib/treasury-contract-service";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";

// Query keys
const TREASURY_QUERY_KEYS = {
  balance: (contractAddress: HexString) =>
    ["treasury", "balance", contractAddress] as const,
  pendingPayouts: (contractAddress: HexString, accountAddress: string) =>
    ["treasury", "pendingPayouts", contractAddress, accountAddress] as const,
  pendingPayoutIds: (contractAddress: HexString, accountAddress: string) =>
    ["treasury", "pendingPayoutIds", contractAddress, accountAddress] as const,
};

export function useTreasuryContract(
  contractAddress: HexString,
  ss58Address?: string
) {
  const { selectedAccount } = usePolkadotExtension();
  const queryClient = useQueryClient();

  // Query for treasury balance using contract's get_balance function
  const balanceQuery = useQuery({
    queryKey: TREASURY_QUERY_KEYS.balance(contractAddress),
    queryFn: async () => {
      return await treasuryContractService.getBalance(contractAddress);
    },
    enabled: !!contractAddress,
    refetchInterval: 30000, // Refetch every 30 seconds
  });

  // Query for pending payouts
  const pendingPayoutsQuery = useQuery({
    queryKey: TREASURY_QUERY_KEYS.pendingPayouts(
      contractAddress,
      selectedAccount?.address || ""
    ),
    queryFn: async () => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }
      return await treasuryContractService.getPendingPayouts(
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
      selectedAccount?.address || ""
    ),
    queryFn: async () => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }
      return await treasuryContractService.getPendingPayoutIds(
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
      return await treasuryContractService.addPayout(
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
          selectedAccount?.address || ""
        ),
      });
      queryClient.invalidateQueries({
        queryKey: TREASURY_QUERY_KEYS.pendingPayoutIds(
          contractAddress,
          selectedAccount?.address || ""
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
      return await treasuryContractService.addPayoutBatch(
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
          selectedAccount?.address || ""
        ),
      });
      queryClient.invalidateQueries({
        queryKey: TREASURY_QUERY_KEYS.pendingPayoutIds(
          contractAddress,
          selectedAccount?.address || ""
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

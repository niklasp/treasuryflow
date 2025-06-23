"use client";

import { useMutation } from "@tanstack/react-query";
import { HexString } from "polkadot-api";
import { deployTreasury } from "@/lib/deploy-treasury";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";

interface DeployTreasuryResult {
  //   ss58Address: string;
  contractAddress: HexString | null;
}

export function useDeployTreasury() {
  const { selectedAccount } = usePolkadotExtension();

  const mutation = useMutation<DeployTreasuryResult, Error>({
    mutationFn: async () => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }
      return await deployTreasury(selectedAccount);
    },
    onSuccess: (data) => {
      console.log("Treasury deployed successfully:", data);
    },
    onError: (error) => {
      console.error("Treasury deployment failed:", error);
    },
  });

  return {
    deployTreasury: mutation.mutate,
    contractAddress: mutation.data?.contractAddress || null,
    isLoading: mutation.isPending,
    error: mutation.error?.message,
    isSuccess: mutation.isSuccess,
    reset: mutation.reset,
  };
}

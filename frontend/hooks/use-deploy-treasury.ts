"use client";

import { useMutation } from "@tanstack/react-query";
import { useMutation as useConvexMutation } from "convex/react";
import { HexString } from "polkadot-api";
import {
  createTreasuryContractService,
  DeployTreasuryResult,
} from "@/lib/treasury-contract-service";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { api } from "@/convex/_generated/api";
import { NetworkId } from "@/lib/treasury-contract-service";

interface Treasurer {
  name: string;
  address: string;
}

interface CreateTreasuryFormValues {
  name: string;
  description: string;
  currencies: string[];
  payoutFrequency: string;
  treasurers: Treasurer[];
  network: NetworkId;
}

// Using DeployTreasuryResult from treasury-contract-service

export function useDeployTreasury() {
  const { selectedAccount } = usePolkadotExtension();
  const createDbTreasury = useConvexMutation(api.treasuries.create);

  const mutation = useMutation<
    DeployTreasuryResult,
    Error,
    CreateTreasuryFormValues
  >({
    mutationFn: async (formData: CreateTreasuryFormValues) => {
      if (!selectedAccount) {
        throw new Error("No account selected");
      }

      // Create service instance for the selected network
      const service = createTreasuryContractService(formData.network);
      const deploymentResult = await service.deploy(selectedAccount);

      if (!deploymentResult) {
        throw new Error("Failed to deploy treasury");
      }
      await createDbTreasury({
        owner: selectedAccount.address,
        name: formData.name,
        description: formData.description,
        contractAddress: deploymentResult.contractAddress,
        ss58Address: deploymentResult.ss58Address,
        network: formData.network,
        currencies: formData.currencies,
        payoutFrequency: formData.payoutFrequency,
        treasurers: formData.treasurers,
      });

      return deploymentResult;
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

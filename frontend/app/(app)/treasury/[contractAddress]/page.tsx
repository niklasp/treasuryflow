"use client";

import { useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Wallet, Users, Clock, RefreshCw, Plus } from "lucide-react";
import { notFound } from "next/navigation";
import { TreasuryBalance } from "@/components/treasury/treasury-balance";
import { TreasuryPayouts } from "@/components/treasury/treasury-payouts";
import { stringifyWithBigInt, trimAddress } from "@/lib/utils";
import { formatBalance } from "@/lib/format-balance";
import { CopyButton } from "@/components/ui/copy-button";
import { useTreasuryContract } from "@/hooks/use-treasury-contract";
import { Button } from "@/components/ui/button";
import { HexString } from "polkadot-api";
import { useEffect, useState } from "react";
import Link from "next/link";
import { CONTRACT_NETWORKS, NetworkId } from "@/lib/treasury-contract-service";

interface TreasuryPageProps {
  params: {
    contractAddress: string;
  };
}

export default function TreasuryPage({ params }: TreasuryPageProps) {
  const [contractAddress, setContractAddress] = useState<string>("");

  useEffect(() => {
    const resolveParams = async () => {
      const resolvedParams = await params;
      setContractAddress(decodeURIComponent(resolvedParams.contractAddress));
    };
    resolveParams();
  }, [params]);

  // Get treasury data from Convex
  const treasury = useQuery(
    api.treasuries.getByContractAddress,
    contractAddress ? { contractAddress } : "skip"
  );

  // Determine the network for this treasury
  const networkId = (treasury?.network as NetworkId) || "PASSET_HUB"; // Default to PASSET_HUB for legacy treasuries

  // Get contract data using the treasury contract hook
  const {
    balance,
    pendingPayouts,
    pendingPayoutIds,
    isLoadingBalance,
    isLoadingPayouts,
    isLoadingPayoutIds,
    balanceError,
    payoutsError,
    payoutIdsError,
    refetchBalance,
    refetchPayouts,
    refetchPayoutIds,
  } = useTreasuryContract(contractAddress as HexString, networkId);

  if (!contractAddress) {
    return <div>Loading...</div>;
  }

  if (treasury === null) {
    notFound();
  }

  if (treasury === undefined) {
    return <div>Loading treasury...</div>;
  }

  return (
    <div className="flex-1">
      <div className="container grid flex-1 gap-4 items-start px-4 py-12 md:px-6">
        <div className="mx-auto w-full max-w-[1200px] space-y-6">
          {/* Header */}
          <div className="space-y-2">
            <h1 className="text-3xl font-bold tracking-tight">
              {treasury.name}
            </h1>
            {treasury.description && (
              <p className="text-lg text-muted-foreground">
                {treasury.description}
              </p>
            )}
          </div>

          {/* Treasury Overview */}
          <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-4">
            <div className="flex flex-col justify-center items-center">
              <Link href={`/new-payout/${contractAddress}`} className="w-full">
                <Button size="lg" className="w-full h-16 text-lg font-semibold">
                  <Plus className="mr-3 w-6 h-6" />
                  Add Payout
                </Button>
              </Link>
              <p className="mt-2 text-xs text-center text-muted-foreground">
                Create a new payout for this treasury
              </p>
            </div>

            <Card>
              <CardHeader className="flex flex-row justify-between items-center pb-2 space-y-0">
                <CardTitle className="text-sm font-medium">
                  Contract Details
                </CardTitle>
                <Wallet className="w-4 h-4 text-primary" />
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex justify-between items-center">
                  <div className="flex-1 mr-2 text-xs break-all text-muted-foreground">
                    <span className="mr-2 font-bold">Network:</span>
                    {
                      treasury.network
                        ? CONTRACT_NETWORKS[treasury.network as NetworkId]
                            ?.name || treasury.network
                        : "Passet Hub" // Default for legacy treasuries
                    }
                  </div>
                </div>
                <div className="flex justify-between items-center">
                  <div className="flex-1 mr-2 font-mono text-xs break-all text-muted-foreground">
                    <span className="mr-2 font-bold">Contract:</span>
                    {trimAddress(treasury.contractAddress, 8)}
                  </div>
                  <CopyButton value={treasury.contractAddress} />
                </div>
                <div className="flex justify-between items-center">
                  <div className="flex-1 mr-2 font-mono text-xs break-all text-muted-foreground">
                    <span className="mr-2 font-bold">SS58:</span>
                    {trimAddress(treasury.ss58Address, 8)}
                  </div>
                  <CopyButton value={treasury.ss58Address} />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row justify-between items-center pb-2 space-y-0">
                <CardTitle className="text-sm font-medium">
                  Treasurers
                </CardTitle>
                <Users className="w-4 h-4 text-accent" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {treasury.treasurers?.length || 0}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row justify-between items-center pb-2 space-y-0">
                <CardTitle className="text-sm font-medium">
                  Treasury Balance
                </CardTitle>
                <Wallet className="w-4 h-4 text-primary" />
              </CardHeader>
              <CardContent>
                {isLoadingBalance ? (
                  <div className="animate-pulse">
                    <div className="w-24 h-8 bg-gray-300 rounded"></div>
                  </div>
                ) : balanceError ? (
                  <div
                    className="text-sm text-destructive"
                    title={
                      balanceError instanceof Error
                        ? balanceError.message
                        : "Unknown error"
                    }
                  >
                    Balance unavailable
                  </div>
                ) : (
                  <div className="text-2xl font-bold">
                    {balance ? formatBalance(balance) : "0 DOT"}
                  </div>
                )}
              </CardContent>
            </Card>
          </div>

          {/* Contract Pending Payouts */}
          <Card>
            <CardHeader className="flex flex-row justify-between items-center pb-4 space-y-0">
              <CardTitle className="flex gap-2 items-center">
                <Clock className="w-5 h-5 text-accent" />
                Pending Payouts
              </CardTitle>
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  refetchBalance();
                  refetchPayouts();
                  refetchPayoutIds();
                }}
                disabled={
                  isLoadingBalance || isLoadingPayouts || isLoadingPayoutIds
                }
              >
                <RefreshCw className="mr-2 w-4 h-4" />
                Refresh
              </Button>
            </CardHeader>
            <CardContent className="space-y-4">
              {isLoadingPayouts || isLoadingPayoutIds ? (
                <div className="py-8 text-center">
                  <div className="mx-auto mb-4 w-8 h-8 rounded-full border-b-2 animate-spin border-primary"></div>
                  <p className="text-muted-foreground">
                    Loading pending payouts...
                  </p>
                </div>
              ) : payoutsError || payoutIdsError ? (
                <div className="py-8 text-center">
                  <p className="mb-2 text-destructive">
                    Failed to load pending payouts
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {payoutsError?.message || payoutIdsError?.message}
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  <div>
                    <h4 className="mb-2 text-sm font-medium text-muted-foreground">
                      Pending Payout IDs
                    </h4>
                    <div className="p-3 rounded-md border bg-muted/20">
                      <pre className="text-xs whitespace-pre-wrap text-muted-foreground">
                        {pendingPayoutIds
                          ? stringifyWithBigInt(pendingPayoutIds)
                          : "No pending payout IDs"}
                      </pre>
                    </div>
                  </div>

                  <div>
                    <h4 className="mb-2 text-sm font-medium text-muted-foreground">
                      Pending Payouts Details
                    </h4>
                    <div className="p-3 rounded-md border bg-muted/20">
                      <pre className="text-xs whitespace-pre-wrap text-muted-foreground">
                        {pendingPayouts
                          ? stringifyWithBigInt(pendingPayouts)
                          : "No pending payouts"}
                      </pre>
                    </div>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Treasury Details */}
          <TreasuryBalance treasuryData={treasury} />
          <TreasuryPayouts treasuryData={treasury} />
        </div>
      </div>
    </div>
  );
}

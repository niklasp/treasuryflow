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
  } = useTreasuryContract(contractAddress as HexString);

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
      <div className="container grid flex-1 items-start gap-4 px-4 py-12 md:px-6">
        <div className="mx-auto w-full max-w-[1200px] space-y-6">
          {/* Header */}
          <div className="space-y-2">
            <h1 className="text-3xl font-bold tracking-tight">
              {treasury.name}
            </h1>
            {treasury.description && (
              <p className="text-muted-foreground text-lg">
                {treasury.description}
              </p>
            )}
          </div>

          {/* Treasury Overview */}
          <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-4">
            <div className="flex flex-col items-center justify-center">
              <Link href={`/new-payout/${contractAddress}`} className="w-full">
                <Button size="lg" className="w-full h-16 text-lg font-semibold">
                  <Plus className="h-6 w-6 mr-3" />
                  Add Payout
                </Button>
              </Link>
              <p className="text-xs text-muted-foreground mt-2 text-center">
                Create a new payout for this treasury
              </p>
            </div>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  Contract Addresses
                </CardTitle>
                <Wallet className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="text-xs font-mono text-muted-foreground break-all flex-1 mr-2">
                    <span className="font-bold mr-2">Contract:</span>
                    {trimAddress(treasury.contractAddress, 8)}
                  </div>
                  <CopyButton value={treasury.contractAddress} />
                </div>
                <div className="flex items-center justify-between">
                  <div className="text-xs font-mono text-muted-foreground break-all flex-1 mr-2">
                    <span className="font-bold mr-2">SS58:</span>
                    {trimAddress(treasury.ss58Address, 8)}
                  </div>
                  <CopyButton value={treasury.ss58Address} />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  Treasurers
                </CardTitle>
                <Users className="h-4 w-4 text-accent" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {treasury.treasurers?.length || 0}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  Treasury Balance
                </CardTitle>
                <Wallet className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent>
                {isLoadingBalance ? (
                  <div className="animate-pulse">
                    <div className="h-8 bg-gray-300 rounded w-24"></div>
                  </div>
                ) : balanceError ? (
                  <div className="text-sm text-destructive">Failed to load</div>
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
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
              <CardTitle className="flex items-center gap-2">
                <Clock className="h-5 w-5 text-accent" />
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
                <RefreshCw className="h-4 w-4 mr-2" />
                Refresh
              </Button>
            </CardHeader>
            <CardContent className="space-y-4">
              {isLoadingPayouts || isLoadingPayoutIds ? (
                <div className="text-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
                  <p className="text-muted-foreground">
                    Loading pending payouts...
                  </p>
                </div>
              ) : payoutsError || payoutIdsError ? (
                <div className="text-center py-8">
                  <p className="text-destructive mb-2">
                    Failed to load pending payouts
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {payoutsError?.message || payoutIdsError?.message}
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  <div>
                    <h4 className="text-sm font-medium text-muted-foreground mb-2">
                      Pending Payout IDs
                    </h4>
                    <div className="rounded-md border bg-muted/20 p-3">
                      <pre className="text-xs text-muted-foreground whitespace-pre-wrap">
                        {pendingPayoutIds
                          ? stringifyWithBigInt(pendingPayoutIds)
                          : "No pending payout IDs"}
                      </pre>
                    </div>
                  </div>

                  <div>
                    <h4 className="text-sm font-medium text-muted-foreground mb-2">
                      Pending Payouts Details
                    </h4>
                    <div className="rounded-md border bg-muted/20 p-3">
                      <pre className="text-xs text-muted-foreground whitespace-pre-wrap">
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

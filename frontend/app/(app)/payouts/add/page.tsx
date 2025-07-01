"use client";

import { useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import Link from "next/link";
import { Wallet, ArrowRight, Plus } from "lucide-react";
import { trimAddress } from "@/lib/utils";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { CONTRACT_NETWORKS, NetworkId } from "@/lib/treasury-contract-service";

export default function AddPayoutPage() {
  const { selectedAccount } = usePolkadotExtension();

  const treasuries = useQuery(
    api.treasuries.listByOwner,
    selectedAccount ? { owner: selectedAccount.address } : "skip"
  );

  if (!selectedAccount) {
    return (
      <div className="flex-1">
        <div className="container grid flex-1 gap-4 items-start px-4 py-12 md:px-6">
          <div className="mx-auto w-full max-w-[800px] space-y-6">
            <Card>
              <CardContent className="py-8 text-center">
                <Wallet className="mx-auto mb-4 w-12 h-12 text-muted-foreground" />
                <h3 className="mb-2 text-lg font-semibold">
                  Connect Your Wallet
                </h3>
                <p className="text-muted-foreground">
                  Please connect your Polkadot wallet to view your treasuries.
                </p>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1">
      <div className="container grid flex-1 gap-4 items-start px-4 py-12 md:px-6">
        <div className="mx-auto w-full max-w-[800px] space-y-6">
          {/* Header */}
          <div className="space-y-2">
            <h1 className="text-3xl font-bold tracking-tight">
              Create New Payout
            </h1>
            <p className="text-lg text-muted-foreground">
              Select a treasury to create a new payout from.
            </p>
          </div>

          {/* Treasury Selection */}
          {treasuries === undefined ? (
            <div className="py-8 text-center">
              <div className="mx-auto mb-4 w-8 h-8 rounded-full border-b-2 animate-spin border-primary"></div>
              <p className="text-muted-foreground">Loading treasuries...</p>
            </div>
          ) : treasuries.length === 0 ? (
            <Card>
              <CardContent className="py-8 text-center">
                <div className="mb-4">
                  <Wallet className="mx-auto mb-4 w-12 h-12 text-muted-foreground" />
                  <h3 className="mb-2 text-lg font-semibold">
                    No Treasuries Found
                  </h3>
                  <p className="text-muted-foreground">
                    You need to create a treasury first before you can add
                    payouts.
                  </p>
                </div>
                <Link href="/create-treasury">
                  <Button>
                    <Plus className="mr-2 w-4 h-4" />
                    Create Treasury
                  </Button>
                </Link>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4">
              {treasuries.map((treasury) => (
                <Card
                  key={treasury._id}
                  className="transition-colors hover:bg-muted/50"
                >
                  <CardHeader>
                    <div className="flex justify-between items-center">
                      <div>
                        <CardTitle className="flex gap-2 items-center">
                          <Wallet className="w-5 h-5 text-primary" />
                          {treasury.name}
                        </CardTitle>
                        {treasury.description && (
                          <CardDescription className="mt-1">
                            {treasury.description}
                          </CardDescription>
                        )}
                      </div>
                      <Link href={`/new-payout/${treasury.contractAddress}`}>
                        <Button>
                          Create Payout
                          <ArrowRight className="ml-2 w-4 h-4" />
                        </Button>
                      </Link>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="grid gap-2 text-sm">
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">Network:</span>
                        <span>
                          {
                            treasury.network
                              ? CONTRACT_NETWORKS[treasury.network as NetworkId]
                                  ?.name || treasury.network
                              : "Passet Hub" // Default for legacy treasuries
                          }
                        </span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">
                          Contract Address:
                        </span>
                        <span className="font-mono text-xs">
                          {trimAddress(treasury.contractAddress, 8)}
                        </span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">
                          Treasurers:
                        </span>
                        <span>{treasury.treasurers?.length || 0}</span>
                      </div>
                      <div className="flex justify-between items-center">
                        <span className="text-muted-foreground">
                          Payout Frequency:
                        </span>
                        <span className="capitalize">
                          {treasury.payoutFrequency || "Not set"}
                        </span>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

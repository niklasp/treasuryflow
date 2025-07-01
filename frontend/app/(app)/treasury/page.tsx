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
import { Badge } from "@/components/ui/badge";
import Link from "next/link";
import { useRouter } from "next/navigation";
import {
  Wallet,
  Plus,
  Users,
  Clock,
  ArrowRight,
  DollarSign,
} from "lucide-react";
import { trimAddress } from "@/lib/utils";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { CopyButton } from "@/components/ui/copy-button";
import { TreasuryCardBalance } from "@/components/treasury/treasury-card-balance";
import { CONTRACT_NETWORKS, NetworkId } from "@/lib/treasury-contract-service";

export default function TreasuryDashboard() {
  const { selectedAccount } = usePolkadotExtension();
  const router = useRouter();

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
        <div className="mx-auto w-full max-w-[1200px] space-y-6">
          {/* Header */}
          <div className="flex justify-between items-center">
            <div className="space-y-2">
              <h1 className="text-3xl font-bold tracking-tight">
                Your Treasuries
              </h1>
              <p className="text-lg text-muted-foreground">
                Manage and monitor your treasury contracts.
              </p>
            </div>
            <Link href="/create-treasury">
              <Button>
                <Plus className="mr-2 w-4 h-4" />
                Create Treasury
              </Button>
            </Link>
          </div>

          {/* Treasury Grid */}
          {treasuries === undefined ? (
            <div className="py-8 text-center">
              <div className="mx-auto mb-4 w-8 h-8 rounded-full border-b-2 animate-spin border-primary"></div>
              <p className="text-muted-foreground">
                Loading your treasuries...
              </p>
            </div>
          ) : treasuries.length === 0 ? (
            <Card>
              <CardContent className="py-12 text-center">
                <div className="mb-6">
                  <Wallet className="mx-auto mb-4 w-16 h-16 text-muted-foreground" />
                  <h3 className="mb-2 text-xl font-semibold">
                    No Treasuries Yet
                  </h3>
                  <p className="mx-auto max-w-md text-muted-foreground">
                    Create your first treasury to start managing payouts and
                    organizing your funds.
                  </p>
                </div>
                <Link href="/create-treasury">
                  <Button>
                    <Plus className="mr-2 w-4 h-4" />
                    Create Your First Treasury
                  </Button>
                </Link>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {treasuries.map((treasury) => (
                <Card
                  key={treasury._id}
                  className="h-full transition-colors cursor-pointer hover:bg-muted/50 group hover:scale-[1.02] transition-transform"
                  onClick={() =>
                    router.push(`/treasury/${treasury.contractAddress}`)
                  }
                >
                  <CardHeader className="pb-3">
                    <div className="flex justify-between items-start">
                      <div className="flex-1 min-w-0">
                        <CardTitle className="flex gap-2 items-center text-lg">
                          <Wallet className="flex-shrink-0 w-5 h-5 text-primary" />
                          <span className="truncate">{treasury.name}</span>
                        </CardTitle>
                        {treasury.description && (
                          <CardDescription className="mt-2 line-clamp-2">
                            {treasury.description}
                          </CardDescription>
                        )}
                      </div>
                      <ArrowRight className="flex-shrink-0 ml-2 w-4 h-4 transition-colors text-muted-foreground group-hover:text-primary" />
                    </div>
                  </CardHeader>

                  <CardContent className="space-y-4">
                    {/* Quick Stats */}
                    <div className="grid grid-cols-2 gap-3">
                      <div className="flex gap-2 items-center">
                        <DollarSign className="w-4 h-4 text-primary" />
                        <div>
                          <p className="text-xs text-muted-foreground">
                            Balance
                          </p>
                          <TreasuryCardBalance
                            contractAddress={treasury.contractAddress as any}
                            networkId={
                              (treasury.network as NetworkId) || "PASSET_HUB"
                            }
                            ss58Address={treasury.ss58Address}
                          />
                        </div>
                      </div>

                      <div className="flex gap-2 items-center">
                        <Users className="w-4 h-4 text-accent" />
                        <div>
                          <p className="text-xs text-muted-foreground">
                            Treasurers
                          </p>
                          <p className="text-sm font-medium">
                            {treasury.treasurers?.length || 0}
                          </p>
                        </div>
                      </div>
                    </div>

                    {/* Network */}
                    <div>
                      <p className="mb-2 text-xs text-muted-foreground">
                        Network
                      </p>
                      <Badge variant="outline" className="text-xs">
                        {
                          treasury.network
                            ? CONTRACT_NETWORKS[treasury.network as NetworkId]
                                ?.name || treasury.network
                            : "Passet Hub" // Default for legacy treasuries
                        }
                      </Badge>
                    </div>

                    {/* Currencies */}
                    {treasury.currencies && treasury.currencies.length > 0 && (
                      <div>
                        <p className="mb-2 text-xs text-muted-foreground">
                          Currencies
                        </p>
                        <div className="flex flex-wrap gap-1">
                          {treasury.currencies.slice(0, 3).map((currency) => (
                            <Badge
                              key={currency}
                              variant="secondary"
                              className="text-xs border bg-muted text-foreground"
                            >
                              {currency}
                            </Badge>
                          ))}
                          {treasury.currencies.length > 3 && (
                            <Badge
                              variant="secondary"
                              className="text-xs border bg-muted text-muted-foreground"
                            >
                              +{treasury.currencies.length - 3}
                            </Badge>
                          )}
                        </div>
                      </div>
                    )}

                    {/* Contract Address */}
                    <div className="pt-2 border-t">
                      <div className="flex justify-between items-center">
                        <div className="flex-1 min-w-0">
                          <p className="text-xs text-muted-foreground">
                            Contract Address
                          </p>
                          <p className="font-mono text-xs text-foreground">
                            {trimAddress(treasury.contractAddress, 8)}
                          </p>
                        </div>
                        <div onClick={(e) => e.preventDefault()}>
                          <CopyButton value={treasury.contractAddress} />
                        </div>
                      </div>
                    </div>

                    {/* Quick Actions */}
                    <div className="pt-2 border-t">
                      <div className="flex gap-2">
                        <Button
                          size="sm"
                          variant="outline"
                          className="flex-1 text-xs"
                          onClick={(e) => {
                            e.stopPropagation();
                            router.push(
                              `/new-payout/${treasury.contractAddress}`
                            );
                          }}
                        >
                          <DollarSign className="mr-1 w-3 h-3" />
                          Add Payout
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          className="px-3"
                          onClick={(e) => e.stopPropagation()}
                        >
                          <ArrowRight className="w-3 h-3" />
                        </Button>
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

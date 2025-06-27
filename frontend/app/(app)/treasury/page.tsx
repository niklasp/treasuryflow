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

export default function TreasuryDashboard() {
  const { selectedAccount } = usePolkadotExtension();

  const treasuries = useQuery(
    api.treasuries.listByOwner,
    selectedAccount ? { owner: selectedAccount.address } : "skip"
  );

  if (!selectedAccount) {
    return (
      <div className="flex-1">
        <div className="container grid flex-1 items-start gap-4 px-4 py-12 md:px-6">
          <div className="mx-auto w-full max-w-[800px] space-y-6">
            <Card>
              <CardContent className="text-center py-8">
                <Wallet className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-lg font-semibold mb-2">
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
      <div className="container grid flex-1 items-start gap-4 px-4 py-12 md:px-6">
        <div className="mx-auto w-full max-w-[1200px] space-y-6">
          {/* Header */}
          <div className="flex items-center justify-between">
            <div className="space-y-2">
              <h1 className="text-3xl font-bold tracking-tight">
                Your Treasuries
              </h1>
              <p className="text-muted-foreground text-lg">
                Manage and monitor your treasury contracts.
              </p>
            </div>
            <Link href="/create-treasury">
              <Button>
                <Plus className="h-4 w-4 mr-2" />
                Create Treasury
              </Button>
            </Link>
          </div>

          {/* Treasury Grid */}
          {treasuries === undefined ? (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
              <p className="text-muted-foreground">
                Loading your treasuries...
              </p>
            </div>
          ) : treasuries.length === 0 ? (
            <Card>
              <CardContent className="text-center py-12">
                <div className="mb-6">
                  <Wallet className="h-16 w-16 text-muted-foreground mx-auto mb-4" />
                  <h3 className="text-xl font-semibold mb-2">
                    No Treasuries Yet
                  </h3>
                  <p className="text-muted-foreground max-w-md mx-auto">
                    Create your first treasury to start managing payouts and
                    organizing your funds.
                  </p>
                </div>
                <Link href="/create-treasury">
                  <Button>
                    <Plus className="h-4 w-4 mr-2" />
                    Create Your First Treasury
                  </Button>
                </Link>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {treasuries.map((treasury) => (
                <Link
                  key={treasury._id}
                  href={`/treasury/${treasury.contractAddress}`}
                  className="block transition-transform hover:scale-[1.02]"
                >
                  <Card className="hover:bg-muted/50 transition-colors h-full cursor-pointer group">
                    <CardHeader className="pb-3">
                      <div className="flex items-start justify-between">
                        <div className="flex-1 min-w-0">
                          <CardTitle className="flex items-center gap-2 text-lg">
                            <Wallet className="h-5 w-5 text-primary flex-shrink-0" />
                            <span className="truncate">{treasury.name}</span>
                          </CardTitle>
                          {treasury.description && (
                            <CardDescription className="mt-2 line-clamp-2">
                              {treasury.description}
                            </CardDescription>
                          )}
                        </div>
                        <ArrowRight className="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors flex-shrink-0 ml-2" />
                      </div>
                    </CardHeader>

                    <CardContent className="space-y-4">
                      {/* Quick Stats */}
                      <div className="grid grid-cols-2 gap-3">
                        <div className="flex items-center gap-2">
                          <DollarSign className="h-4 w-4 text-primary" />
                          <div>
                            <p className="text-xs text-muted-foreground">
                              Balance
                            </p>
                            <TreasuryCardBalance
                              contractAddress={treasury.contractAddress as any}
                              ss58Address={treasury.ss58Address}
                            />
                          </div>
                        </div>

                        <div className="flex items-center gap-2">
                          <Users className="h-4 w-4 text-accent" />
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

                      {/* Currencies */}
                      {treasury.currencies &&
                        treasury.currencies.length > 0 && (
                          <div>
                            <p className="text-xs text-muted-foreground mb-2">
                              Currencies
                            </p>
                            <div className="flex flex-wrap gap-1">
                              {treasury.currencies
                                .slice(0, 3)
                                .map((currency) => (
                                  <Badge
                                    key={currency}
                                    variant="secondary"
                                    className="text-xs bg-muted text-foreground border"
                                  >
                                    {currency}
                                  </Badge>
                                ))}
                              {treasury.currencies.length > 3 && (
                                <Badge
                                  variant="secondary"
                                  className="text-xs bg-muted text-muted-foreground border"
                                >
                                  +{treasury.currencies.length - 3}
                                </Badge>
                              )}
                            </div>
                          </div>
                        )}

                      {/* Contract Address */}
                      <div className="pt-2 border-t">
                        <div className="flex items-center justify-between">
                          <div className="min-w-0 flex-1">
                            <p className="text-xs text-muted-foreground">
                              Contract Address
                            </p>
                            <p className="text-xs font-mono text-foreground">
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
                          <Link
                            href={`/new-payout/${treasury.contractAddress}`}
                            onClick={(e) => e.stopPropagation()}
                            className="flex-1"
                          >
                            <Button
                              size="sm"
                              variant="outline"
                              className="w-full text-xs"
                            >
                              <DollarSign className="h-3 w-3 mr-1" />
                              Add Payout
                            </Button>
                          </Link>
                          <Button
                            size="sm"
                            variant="outline"
                            className="px-3"
                            onClick={(e) => e.stopPropagation()}
                          >
                            <ArrowRight className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </Link>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

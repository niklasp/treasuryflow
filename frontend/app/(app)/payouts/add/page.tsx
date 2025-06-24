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

export default function AddPayoutPage() {
  const treasuries = useQuery(api.treasuries.list);

  return (
    <div className="flex-1">
      <div className="container grid flex-1 items-start gap-4 px-4 py-12 md:px-6">
        <div className="mx-auto w-full max-w-[800px] space-y-6">
          {/* Header */}
          <div className="space-y-2">
            <h1 className="text-3xl font-bold tracking-tight">
              Create New Payout
            </h1>
            <p className="text-muted-foreground text-lg">
              Select a treasury to create a new payout from.
            </p>
          </div>

          {/* Treasury Selection */}
          {treasuries === undefined ? (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
              <p className="text-muted-foreground">Loading treasuries...</p>
            </div>
          ) : treasuries.length === 0 ? (
            <Card className="bg-card text-card-foreground">
              <CardContent className="text-center py-8">
                <div className="mb-4">
                  <Wallet className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                  <h3 className="text-lg font-semibold mb-2">
                    No Treasuries Found
                  </h3>
                  <p className="text-muted-foreground">
                    You need to create a treasury first before you can add
                    payouts.
                  </p>
                </div>
                <Link href="/treasury/create">
                  <Button>
                    <Plus className="h-4 w-4 mr-2" />
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
                  className="bg-card text-card-foreground hover:bg-muted/50 transition-colors"
                >
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle className="flex items-center gap-2">
                          <Wallet className="h-5 w-5 text-primary" />
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
                          <ArrowRight className="h-4 w-4 ml-2" />
                        </Button>
                      </Link>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="grid gap-2 text-sm">
                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">
                          Contract Address:
                        </span>
                        <span className="font-mono text-xs">
                          {trimAddress(treasury.contractAddress, 8)}
                        </span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-muted-foreground">
                          Treasurers:
                        </span>
                        <span>{treasury.treasurers?.length || 0}</span>
                      </div>
                      <div className="flex items-center justify-between">
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

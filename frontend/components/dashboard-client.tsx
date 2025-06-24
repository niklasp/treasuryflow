"use client";

import Link from "next/link";
import { Wallet, Zap, Clock, TrendingUp, PlusCircle } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { TreasuryBalance } from "./treasury/treasury-balance";
import { TreasuryPayouts } from "./treasury/treasury-payouts";

interface DashboardContentProps {
  treasuryData: {
    totalAmount: number;
    nextPayoutDays: number;
    pendingPayouts: number;
    chartData: Array<{ name: string; amount: number }>;
    payouts: Array<{
      id: number;
      recipient: string;
      amount: number;
      unit: string;
      date: string;
    }>;
  };
}

export function DashboardContent({ treasuryData }: DashboardContentProps) {
  return (
    <div className="p-4 md:p-6">
      <div className="grid gap-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">
              Treasury Dashboard
            </h1>
            <p className="text-muted-foreground">
              Monitor your treasury performance and manage payouts.
            </p>
          </div>
          <Link href="/payouts/add">
            <Button>
              <PlusCircle className="mr-2 h-4 w-4" />
              Add Payout
            </Button>
          </Link>
        </div>
        <div className="grid gap-6 md:grid-cols-3">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">
                Total Amount
              </CardTitle>
              <Wallet className="h-4 w-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                ${treasuryData.totalAmount.toLocaleString()}
              </div>
              <p className="flex items-center text-xs text-primary">
                <TrendingUp className="mr-1 h-3 w-3" />
                +12% from last month
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">Next Payout</CardTitle>
              <Clock className="h-4 w-4 text-accent" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {treasuryData.nextPayoutDays} days
              </div>
              <p className="text-xs text-muted-foreground">
                Scheduled for April 22, 2025
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">
                Pending Payouts
              </CardTitle>
              <Zap className="h-4 w-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {treasuryData.pendingPayouts}
              </div>
              <p className="text-xs text-muted-foreground">Total: $28,500</p>
            </CardContent>
          </Card>
        </div>
        <TreasuryBalance treasuryData={treasuryData} />
        <TreasuryPayouts treasuryData={treasuryData} />
      </div>
    </div>
  );
}

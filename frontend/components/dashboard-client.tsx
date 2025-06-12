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
    <div className="p-4 md:p-6 dot-pattern">
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
            <Button className="primary-gradient hover:primary-gradient-hover glow">
              <PlusCircle className="mr-2 h-4 w-4" />
              Add Payout
            </Button>
          </Link>
        </div>
        <div className="grid gap-6 md:grid-cols-3">
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">
                Total Amount
              </CardTitle>
              <Wallet className="h-4 w-4 text-purple-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                ${treasuryData.totalAmount.toLocaleString()}
              </div>
              <p className="flex items-center text-xs text-green-500">
                <TrendingUp className="mr-1 h-3 w-3" />
                +12% from last month
              </p>
            </CardContent>
          </Card>
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">Next Payout</CardTitle>
              <Clock className="h-4 w-4 text-indigo-500" />
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
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">
                Pending Payouts
              </CardTitle>
              <Zap className="h-4 w-4 text-violet-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {treasuryData.pendingPayouts}
              </div>
              <p className="text-xs text-muted-foreground">Total: $28,500</p>
            </CardContent>
          </Card>
        </div>
        <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
          <CardHeader>
            <CardTitle>Treasury Balance</CardTitle>
            <CardDescription>6-month treasury balance history</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="h-[300px]">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart
                  data={treasuryData.chartData}
                  margin={{
                    top: 10,
                    right: 30,
                    left: 0,
                    bottom: 0,
                  }}
                >
                  <CartesianGrid
                    strokeDasharray="3 3"
                    stroke="rgba(255,255,255,0.1)"
                  />
                  <XAxis dataKey="name" stroke="rgba(255,255,255,0.5)" />
                  <YAxis stroke="rgba(255,255,255,0.5)" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "rgba(0,0,0,0.8)",
                      borderRadius: "0.75rem",
                      border: "1px solid rgba(255,255,255,0.1)",
                      color: "#fff",
                    }}
                  />
                  <Area
                    type="monotone"
                    dataKey="amount"
                    stroke="#6b46c1"
                    fill="url(#colorGradient)"
                    fillOpacity={0.6}
                  />
                  <defs>
                    <linearGradient
                      id="colorGradient"
                      x1="0"
                      y1="0"
                      x2="0"
                      y2="1"
                    >
                      <stop offset="5%" stopColor="#6b46c1" stopOpacity={0.8} />
                      <stop
                        offset="95%"
                        stopColor="#6b46c1"
                        stopOpacity={0.1}
                      />
                    </linearGradient>
                  </defs>
                </AreaChart>
              </ResponsiveContainer>
            </div>
          </CardContent>
        </Card>
        <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>Recent Payouts</CardTitle>
              <CardDescription>A list of your recent payouts</CardDescription>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="border-white/5 bg-black/20 hover:bg-black/40"
            >
              View All
            </Button>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow className="border-white/5">
                  <TableHead>Recipient</TableHead>
                  <TableHead>Amount</TableHead>
                  <TableHead>Date</TableHead>
                  <TableHead className="text-right">Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {treasuryData.payouts.map((payout) => (
                  <TableRow key={payout.id} className="border-white/5">
                    <TableCell className="font-medium">
                      {payout.recipient}
                    </TableCell>
                    <TableCell>
                      {payout.unit} {payout.amount.toLocaleString()}
                    </TableCell>
                    <TableCell>
                      {new Date(payout.date).toLocaleDateString()}
                    </TableCell>
                    <TableCell className="text-right">
                      <span className="inline-flex items-center rounded-full bg-green-950/50 border border-green-500/20 px-2.5 py-0.5 text-xs font-medium text-green-400">
                        Completed
                      </span>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

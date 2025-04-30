"use client";

import { useState } from "react";
import Link from "next/link";
import {
  CalendarDays,
  CreditCard,
  Menu,
  Search,
  User,
  Wallet,
  Zap,
  Clock,
  TrendingUp,
  BarChart4,
  PlusCircle,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
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

interface DashboardClientProps {
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

export function DashboardClient({ treasuryData }: DashboardClientProps) {
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);

  return (
    <div className="flex min-h-screen flex-col">
      <header className="sticky top-0 z-10 flex h-16 items-center gap-4 border-b border-white/5 bg-background/80 backdrop-blur-md px-4 md:px-6">
        <Button
          variant="outline"
          size="icon"
          className="md:hidden"
          onClick={() => setIsSidebarOpen(!isSidebarOpen)}
        >
          <Menu className="h-5 w-5" />
          <span className="sr-only">Toggle sidebar</span>
        </Button>
        <div className="flex items-center gap-2">
          <Link href="/">
            <span className="text-2xl mr-2">🪼</span>
            <span className="text-xl font-semibold tracking-tight">
              TreasuryFlow
            </span>
          </Link>
        </div>
        <div className="ml-auto flex items-center gap-4">
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                className="rounded-full border-white/5 bg-black/20"
              >
                <User className="h-5 w-5 text-purple-500" />
                <span className="sr-only">Toggle user menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              className="border-white/5 bg-black/80 backdrop-blur-md"
            >
              <DropdownMenuLabel>My Account</DropdownMenuLabel>
              <DropdownMenuSeparator className="bg-white/5" />
              <DropdownMenuItem>Settings</DropdownMenuItem>
              <DropdownMenuItem>Support</DropdownMenuItem>
              <DropdownMenuSeparator className="bg-white/5" />
              <DropdownMenuItem>
                <Link href="/" className="flex w-full">
                  Logout
                </Link>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>
      <div className="flex flex-1">
        <aside
          className={`${
            isSidebarOpen ? "flex" : "hidden"
          } w-64 flex-col border-r border-white/5 bg-black/20 backdrop-blur-md md:flex`}
        >
          <nav className="grid gap-2 p-4 text-sm">
            <Link href="/dashboard">
              <Button
                variant="ghost"
                className="flex w-full items-center justify-start gap-3 rounded-lg bg-gradient-to-r from-purple-900/20 to-indigo-900/20 px-3 py-2 text-primary-foreground"
              >
                <BarChart4 className="h-4 w-4 text-purple-500" />
                Dashboard
              </Button>
            </Link>
            <Link href="/create-treasury">
              <Button
                variant="ghost"
                className="flex w-full items-center justify-start gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-colors hover:text-foreground hover:bg-white/5"
              >
                <Wallet className="h-4 w-4 text-green-500" />
                Create Treasury
              </Button>
            </Link>
            <Button
              variant="ghost"
              className="flex items-center justify-start gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-colors hover:text-foreground hover:bg-white/5"
              disabled
            >
              <CreditCard className="h-4 w-4 text-rose-500" />
              Payouts
            </Button>
            <Button
              variant="ghost"
              className="flex items-center justify-start gap-3 rounded-lg px-3 py-2 text-muted-foreground transition-colors hover:text-foreground hover:bg-white/5"
              disabled
            >
              <CalendarDays className="h-4 w-4 text-blue-500" />
              Schedule
            </Button>
          </nav>
        </aside>
        <main className="flex-1 overflow-auto p-4 md:p-6 dot-pattern">
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
              <Button className="primary-gradient hover:primary-gradient-hover glow">
                <PlusCircle className="mr-2 h-4 w-4" />
                Add Payout
              </Button>
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
                  <CardTitle className="text-sm font-medium">
                    Next Payout
                  </CardTitle>
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
                  <p className="text-xs text-muted-foreground">
                    Total: $28,500
                  </p>
                </CardContent>
              </Card>
            </div>
            <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
              <CardHeader>
                <CardTitle>Treasury Balance</CardTitle>
                <CardDescription>
                  6-month treasury balance history
                </CardDescription>
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
                          <stop
                            offset="5%"
                            stopColor="#6b46c1"
                            stopOpacity={0.8}
                          />
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
                  <CardDescription>
                    A list of your recent payouts
                  </CardDescription>
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
        </main>
      </div>
    </div>
  );
}

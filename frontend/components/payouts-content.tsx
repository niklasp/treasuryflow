"use client";

import { useState } from "react";
import Link from "next/link";
import {
  PlusCircle,
  Search,
  Filter,
  Download,
  Clock,
  CheckCircle,
  AlertCircle,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

interface PayoutsContentProps {
  payoutsData: {
    totalPayouts: number;
    totalAmount: number;
    pendingAmount: number;
    payouts: Array<{
      id: number;
      recipient: string;
      amount: number;
      unit: string;
      date: string;
      status: string;
      description: string;
    }>;
  };
}

export function PayoutsContent({ payoutsData }: PayoutsContentProps) {
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");

  const filteredPayouts = payoutsData.payouts.filter((payout) => {
    const matchesSearch =
      payout.recipient.toLowerCase().includes(searchTerm.toLowerCase()) ||
      payout.description.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesStatus =
      statusFilter === "all" || payout.status === statusFilter;
    return matchesSearch && matchesStatus;
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "completed":
        return <CheckCircle className="h-4 w-4 text-green-500" />;
      case "pending":
        return <Clock className="h-4 w-4 text-yellow-500" />;
      case "failed":
        return <AlertCircle className="h-4 w-4 text-red-500" />;
      default:
        return <Clock className="h-4 w-4 text-gray-500" />;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "completed":
        return (
          <Badge className="bg-green-950/50 border-green-500/20 text-green-400 hover:bg-green-950/70">
            Completed
          </Badge>
        );
      case "pending":
        return (
          <Badge className="bg-yellow-950/50 border-yellow-500/20 text-yellow-400 hover:bg-yellow-950/70">
            Pending
          </Badge>
        );
      case "failed":
        return (
          <Badge className="bg-red-950/50 border-red-500/20 text-red-400 hover:bg-red-950/70">
            Failed
          </Badge>
        );
      default:
        return <Badge variant="secondary">{status}</Badge>;
    }
  };

  return (
    <div className="p-4 md:p-6 dot-pattern">
      <div className="grid gap-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Payouts</h1>
            <p className="text-muted-foreground">
              Manage and track all your treasury payouts.
            </p>
          </div>
          <Link href="/payouts/add">
            <Button className="primary-gradient hover:primary-gradient-hover glow">
              <PlusCircle className="mr-2 h-4 w-4" />
              Add Payout
            </Button>
          </Link>
        </div>

        {/* Stats Cards */}
        <div className="grid gap-6 md:grid-cols-3">
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">
                Total Payouts
              </CardTitle>
              <CheckCircle className="h-4 w-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {payoutsData.totalPayouts}
              </div>
              <p className="text-xs text-muted-foreground">All time</p>
            </CardContent>
          </Card>
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">
                Total Amount
              </CardTitle>
              <PlusCircle className="h-4 w-4 text-purple-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                ${payoutsData.totalAmount.toLocaleString()}
              </div>
              <p className="text-xs text-muted-foreground">All time payouts</p>
            </CardContent>
          </Card>
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium">
                Pending Amount
              </CardTitle>
              <Clock className="h-4 w-4 text-yellow-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                ${payoutsData.pendingAmount.toLocaleString()}
              </div>
              <p className="text-xs text-muted-foreground">
                Awaiting processing
              </p>
            </CardContent>
          </Card>
        </div>

        {/* Payouts Table */}
        <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>All Payouts</CardTitle>
                <CardDescription>
                  A complete list of all treasury payouts
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  className="border-white/5 bg-black/20 hover:bg-black/40"
                >
                  <Download className="mr-2 h-4 w-4" />
                  Export
                </Button>
              </div>
            </div>

            {/* Filters */}
            <div className="flex items-center gap-4 pt-4">
              <div className="relative flex-1 max-w-sm">
                <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search payouts..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-8 border-white/5 bg-black/20 focus-visible:ring-primary"
                />
              </div>
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-[180px] border-white/5 bg-black/20 focus:ring-primary">
                  <Filter className="mr-2 h-4 w-4" />
                  <SelectValue placeholder="Filter by status" />
                </SelectTrigger>
                <SelectContent className="border-white/5 bg-black/80 backdrop-blur-md">
                  <SelectItem value="all">All Status</SelectItem>
                  <SelectItem value="completed">Completed</SelectItem>
                  <SelectItem value="pending">Pending</SelectItem>
                  <SelectItem value="failed">Failed</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow className="border-white/5">
                  <TableHead>Recipient</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Amount</TableHead>
                  <TableHead>Date</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredPayouts.map((payout) => (
                  <TableRow key={payout.id} className="border-white/5">
                    <TableCell className="font-medium">
                      {payout.recipient}
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {payout.description}
                    </TableCell>
                    <TableCell>
                      {payout.unit} {payout.amount.toLocaleString()}
                    </TableCell>
                    <TableCell>
                      {new Date(payout.date).toLocaleDateString()}
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        {getStatusIcon(payout.status)}
                        {getStatusBadge(payout.status)}
                      </div>
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-muted-foreground hover:text-foreground"
                      >
                        View
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>

            {filteredPayouts.length === 0 && (
              <div className="text-center py-8">
                <p className="text-muted-foreground">
                  No payouts found matching your criteria.
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

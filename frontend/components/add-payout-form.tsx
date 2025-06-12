"use client";

import type React from "react";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import {
  ArrowLeft,
  Check,
  Plus,
  Upload,
  X,
  DollarSign,
  FileText,
  Tag,
  User,
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
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const labelingCurrencies = [
  { id: "USD", name: "USD", icon: "$" },
  { id: "EUR", name: "EUR", icon: "€" },
  { id: "GBP", name: "GBP", icon: "£" },
];

const paymentCurrencies = [
  { id: "DOT", name: "DOT", icon: "🟣" },
  { id: "USDC", name: "USDC", icon: "🔵" },
  { id: "USDT", name: "USDT", icon: "🟢" },
  { id: "ETH", name: "ETH", icon: "🔷" },
];

const recipients = [
  {
    id: "1",
    name: "Vendor A",
    address: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  },
  {
    id: "2",
    name: "Contractor B",
    address: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
  },
  {
    id: "3",
    name: "Supplier C",
    address: "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy",
  },
];

export function AddPayoutForm() {
  const router = useRouter();
  const [formData, setFormData] = useState({
    recipient: "",
    labelingCurrency: "USD",
    paymentAmount: "",
    paymentCurrency: "DOT",
    reason: "",
    tags: [] as string[],
    attachedFiles: [] as File[],
  });

  const [newTag, setNewTag] = useState("");

  const handleInputChange = (field: string, value: string) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  const addTag = () => {
    if (newTag.trim() && !formData.tags.includes(newTag.trim())) {
      setFormData((prev) => ({
        ...prev,
        tags: [...prev.tags, newTag.trim()],
      }));
      setNewTag("");
    }
  };

  const removeTag = (tagToRemove: string) => {
    setFormData((prev) => ({
      ...prev,
      tags: prev.tags.filter((tag) => tag !== tagToRemove),
    }));
  };

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(event.target.files || []);
    setFormData((prev) => ({
      ...prev,
      attachedFiles: [...prev.attachedFiles, ...files],
    }));
  };

  const removeFile = (fileToRemove: File) => {
    setFormData((prev) => ({
      ...prev,
      attachedFiles: prev.attachedFiles.filter((file) => file !== fileToRemove),
    }));
  };

  const handleSubmit = (payNow: boolean) => {
    console.log("Form submitted:", { ...formData, payNow });
    router.push("/dashboard");
  };

  const selectedRecipient = recipients.find((r) => r.id === formData.recipient);
  const selectedPaymentCurrency = paymentCurrencies.find(
    (c) => c.id === formData.paymentCurrency
  );

  return (
    <>
      <div className="flex items-center gap-4">
        <Link href="/payouts">
          <Button
            variant="outline"
            size="icon"
            className="border-white/5 bg-black/20 hover:bg-black/40"
          >
            <ArrowLeft className="h-4 w-4" />
            <span className="sr-only">Back</span>
          </Button>
        </Link>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">
            Create a new payout
          </h1>
          <p className="text-muted-foreground">
            Create a new one-time, recurring or vested payout from your
            treasury.
          </p>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Main Form */}
        <div className="lg:col-span-2">
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <DollarSign className="h-5 w-5 text-green-500" />
                Payment Details
              </CardTitle>
              <CardDescription>
                Fill in the payment information below.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Recipient Selection */}
              <div className="space-y-2">
                <Label className="flex items-center gap-2">
                  <User className="h-4 w-4 text-blue-500" />
                  Select a Recipient
                </Label>
                <Select
                  value={formData.recipient}
                  onValueChange={(value) =>
                    handleInputChange("recipient", value)
                  }
                >
                  <SelectTrigger className="border-white/5 bg-black/20 focus:ring-primary">
                    <SelectValue placeholder="Find or add new recipient" />
                  </SelectTrigger>
                  <SelectContent className="border-white/5 bg-black/80 backdrop-blur-md">
                    {recipients.map((recipient) => (
                      <SelectItem key={recipient.id} value={recipient.id}>
                        <div className="flex flex-col">
                          <span className="font-medium">{recipient.name}</span>
                          <span className="text-xs text-muted-foreground truncate max-w-[200px]">
                            {recipient.address}
                          </span>
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Currency Labeling */}
              <div className="space-y-2">
                <Label className="flex items-center gap-2">
                  Currency (labeling)
                  <div className="h-4 w-4 rounded-full bg-muted-foreground/20 flex items-center justify-center">
                    <span className="text-xs">?</span>
                  </div>
                </Label>
                <Select
                  value={formData.labelingCurrency}
                  onValueChange={(value) =>
                    handleInputChange("labelingCurrency", value)
                  }
                >
                  <SelectTrigger className="border-white/5 bg-black/20 focus:ring-primary">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent className="border-white/5 bg-black/80 backdrop-blur-md">
                    {labelingCurrencies.map((currency) => (
                      <SelectItem key={currency.id} value={currency.id}>
                        <div className="flex items-center gap-2">
                          <span>{currency.icon}</span>
                          <span>{currency.name}</span>
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Payment Amount */}
              <div className="space-y-2">
                <Label>Payment Amount</Label>
                <div className="flex">
                  <Input
                    type="number"
                    placeholder="0"
                    value={formData.paymentAmount}
                    onChange={(e) =>
                      handleInputChange("paymentAmount", e.target.value)
                    }
                    className="border-white/5 bg-black/20 focus-visible:ring-primary rounded-r-none"
                  />
                  <div className="flex items-center px-3 border border-l-0 border-white/5 bg-black/20 rounded-r-md">
                    <span className="text-sm text-muted-foreground">
                      {formData.labelingCurrency}
                    </span>
                  </div>
                </div>
              </div>

              {/* Payment Currency Selection */}
              <div className="space-y-3">
                <Label>Choose the payment currency on Polkadot Asset Hub</Label>
                <div className="flex flex-wrap gap-2">
                  {paymentCurrencies.map((currency) => (
                    <Button
                      key={currency.id}
                      type="button"
                      variant={
                        formData.paymentCurrency === currency.id
                          ? "default"
                          : "outline"
                      }
                      size="sm"
                      onClick={() =>
                        handleInputChange("paymentCurrency", currency.id)
                      }
                      className={`border-white/5 ${
                        formData.paymentCurrency === currency.id
                          ? "primary-gradient hover:primary-gradient-hover"
                          : "bg-black/20 hover:bg-black/40"
                      }`}
                    >
                      <span className="mr-2">{currency.icon}</span>
                      {currency.name}
                    </Button>
                  ))}
                  {/* <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="border-white/5 bg-black/20 hover:bg-black/40"
                  >
                    ...
                  </Button> */}
                </div>
              </div>

              {/* Reason */}
              <div className="space-y-2">
                <Label className="flex items-center gap-2">
                  <FileText className="h-4 w-4 text-purple-500" />
                  Reason
                </Label>
                <Textarea
                  placeholder="Reason (optional)"
                  value={formData.reason}
                  onChange={(e) => handleInputChange("reason", e.target.value)}
                  className="border-white/5 bg-black/20 focus-visible:ring-primary min-h-[80px]"
                />
              </div>

              {/* Tags */}
              <div className="space-y-3">
                <Label className="flex items-center gap-2">
                  <Tag className="h-4 w-4 text-yellow-500" />
                  Tags
                </Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Select or add a tag"
                    value={newTag}
                    onChange={(e) => setNewTag(e.target.value)}
                    onKeyPress={(e) =>
                      e.key === "Enter" && (e.preventDefault(), addTag())
                    }
                    className="border-white/5 bg-black/20 focus-visible:ring-primary"
                  />
                  <Button
                    type="button"
                    onClick={addTag}
                    variant="outline"
                    size="icon"
                    className="border-white/5 bg-black/20 hover:bg-black/40"
                  >
                    <Plus className="h-4 w-4" />
                  </Button>
                </div>
                {formData.tags.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {formData.tags.map((tag) => (
                      <Badge
                        key={tag}
                        variant="secondary"
                        className="bg-black/40 text-foreground border border-white/10"
                      >
                        {tag}
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={() => removeTag(tag)}
                          className="ml-1 h-auto p-0 text-muted-foreground hover:text-foreground"
                        >
                          <X className="h-3 w-3" />
                        </Button>
                      </Badge>
                    ))}
                  </div>
                )}
              </div>

              {/* Attached Files */}
              <div className="space-y-3">
                <Label>Attached files</Label>
                {formData.attachedFiles.length === 0 ? (
                  <p className="text-sm text-muted-foreground">
                    No file attached yet.
                  </p>
                ) : (
                  <div className="space-y-2">
                    {formData.attachedFiles.map((file, index) => (
                      <div
                        key={index}
                        className="flex items-center justify-between p-2 rounded-md bg-black/20 border border-white/5"
                      >
                        <span className="text-sm">{file.name}</span>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={() => removeFile(file)}
                          className="h-auto p-1 text-muted-foreground hover:text-destructive"
                        >
                          <X className="h-3 w-3" />
                        </Button>
                      </div>
                    ))}
                  </div>
                )}
                <div>
                  <input
                    type="file"
                    multiple
                    onChange={handleFileUpload}
                    className="hidden"
                    id="file-upload"
                  />
                  <Label htmlFor="file-upload">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="border-white/5 bg-black/20 hover:bg-black/40 cursor-pointer"
                      asChild
                    >
                      <span>
                        <Upload className="mr-2 h-4 w-4" />+ Add a file
                      </span>
                    </Button>
                  </Label>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Summary Panel */}
        <div className="lg:col-span-1">
          <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden sticky top-6">
            <CardHeader>
              <CardTitle>Direct Payment</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Recipient */}
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-full bg-muted-foreground/20 flex items-center justify-center">
                  <User className="h-4 w-4" />
                </div>
                <div>
                  <p className="text-sm font-medium">Recipient</p>
                  <p className="text-xs text-muted-foreground">
                    {selectedRecipient
                      ? selectedRecipient.name
                      : "Not selected"}
                  </p>
                </div>
              </div>

              {/* Currency Labeling */}
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-full bg-green-500/20 flex items-center justify-center">
                  <Check className="h-4 w-4 text-green-500" />
                </div>
                <div>
                  <p className="text-sm font-medium">Currency (labeling)</p>
                  <p className="text-xs text-muted-foreground">
                    {formData.labelingCurrency}
                  </p>
                </div>
              </div>

              {/* Payment Amount */}
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-full bg-muted-foreground/20 flex items-center justify-center">
                  <DollarSign className="h-4 w-4" />
                </div>
                <div>
                  <p className="text-sm font-medium">Payment Amount</p>
                  <p className="text-xs text-muted-foreground">
                    {formData.paymentAmount || "0"} {formData.labelingCurrency}
                  </p>
                </div>
              </div>

              {/* Payment Currency */}
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded-full bg-green-500/20 flex items-center justify-center">
                  <Check className="h-4 w-4 text-green-500" />
                </div>
                <div>
                  <p className="text-sm font-medium">Payment Currency</p>
                  <p className="text-xs text-muted-foreground">
                    {selectedPaymentCurrency?.name} on Ethereum
                  </p>
                </div>
              </div>

              {/* Action Buttons */}
              <div className="pt-4 space-y-2">
                <Button
                  onClick={() => handleSubmit(false)}
                  className="w-full primary-gradient hover:primary-gradient-hover glow"
                >
                  Pay with next batch
                </Button>
                <Button
                  onClick={() => handleSubmit(true)}
                  variant="outline"
                  className="w-full border-white/5 bg-black/20 hover:bg-black/40"
                >
                  Pay Now
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </>
  );
}

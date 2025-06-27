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
import { useForm } from "react-hook-form";
import {
  Form,
  FormField,
  FormItem,
  FormLabel,
  FormControl,
  FormMessage,
} from "@/components/ui/form";
import { cn } from "@/lib/utils";
import { useTreasuryContract } from "@/hooks/use-treasury-contract";
import { HexString } from "polkadot-api";
import { toast } from "sonner";
import { Loader2 } from "lucide-react";
import { useQuery } from "convex/react";
import { api } from "@/convex/_generated/api";
import { Id } from "@/convex/_generated/dataModel";
import { trimAddress } from "@/lib/utils";
import { IconNavigationDollar } from "@tabler/icons-react";

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

interface AddPayoutFormValues {
  recipient: string;
  labelingCurrency: string;
  paymentAmount: string;
  paymentCurrency: string;
  reason: string;
  tags: string[];
  attachedFiles: File[];
}

interface AddPayoutFormProps {
  contractAddress: HexString;
}

export function AddPayoutForm({ contractAddress }: AddPayoutFormProps) {
  const router = useRouter();
  const { addPayout, isAddingPayout, addPayoutError } =
    useTreasuryContract(contractAddress);

  const treasury = useQuery(api.treasuries.getByContractAddress, {
    contractAddress: contractAddress as Id<"treasuries">,
  });

  const form = useForm<AddPayoutFormValues>({
    defaultValues: {
      recipient: "",
      labelingCurrency: "",
      paymentAmount: "",
      paymentCurrency: "",
      reason: "",
      tags: [],
      attachedFiles: [],
    },
    mode: "onChange",
  });
  const { control, handleSubmit, setValue, watch, formState } = form;
  const { errors, isValid } = formState;
  const [newTag, setNewTag] = useState("");

  const values = watch();

  async function onSubmit(data: AddPayoutFormValues, payNow: boolean) {
    try {
      const selectedRecipient = recipients.find((r) => r.id === data.recipient);
      if (!selectedRecipient) {
        toast.error("Please select a valid recipient");
        return;
      }

      // Convert amount to bigint (assuming the amount is in the smallest unit)
      const amountBigInt = BigInt(
        Math.floor(parseFloat(data.paymentAmount) * 1e12)
      ); // Convert to planck (12 decimals for DOT)

      // Format recipient address as hex for contract
      const recipientHex = `0x${Buffer.from(selectedRecipient.address, "utf8").toString("hex").padStart(40, "0")}`;

      await addPayout({
        to: recipientHex,
        amount: amountBigInt,
      });

      toast.success("Payout added successfully!");
      router.push(`/treasury/${contractAddress}`);
    } catch (error) {
      console.error("Failed to add payout:", error);
      toast.error("Failed to add payout. Please try again.");
    }
  }

  function addTag() {
    if (!newTag.trim() || values.tags.includes(newTag.trim())) return;
    setValue("tags", [...values.tags, newTag.trim()]);
    setNewTag("");
  }

  function removeTag(tagToRemove: string) {
    setValue(
      "tags",
      values.tags.filter((tag) => tag !== tagToRemove)
    );
  }

  function handleFileUpload(event: React.ChangeEvent<HTMLInputElement>) {
    const files = Array.from(event.target.files || []);
    setValue("attachedFiles", [...values.attachedFiles, ...files]);
  }

  function removeFile(fileToRemove: File) {
    setValue(
      "attachedFiles",
      values.attachedFiles.filter((file) => file !== fileToRemove)
    );
  }

  const selectedRecipient = recipients.find((r) => r.id === values.recipient);
  const selectedPaymentCurrency = paymentCurrencies.find(
    (c) => c.id === values.paymentCurrency
  );

  // Determine summary panel state
  const hasError =
    !!errors.recipient ||
    !!errors.labelingCurrency ||
    !!errors.paymentAmount ||
    !!errors.paymentCurrency;
  const isComplete =
    values.recipient &&
    values.labelingCurrency &&
    values.paymentAmount &&
    values.paymentCurrency &&
    !hasError;
  let summaryBg = "bg-muted";
  if (hasError) summaryBg = "border-red-400";
  else if (isComplete) summaryBg = "border-green-400";

  // Field-level checks for summary icons
  const isRecipientValid = !!values.recipient;
  const isPaymentAmountValid =
    !!values.paymentAmount && Number(values.paymentAmount) > 0;
  const isLabelingCurrencyValid = !!values.labelingCurrency;
  const isPaymentCurrencyValid = !!values.paymentCurrency;

  return (
    <Form {...form}>
      <form
        onSubmit={handleSubmit((data) => onSubmit(data, false))}
        className="h-full"
      >
        <div className="flex flex-col h-full">
          <div className="flex items-center gap-4 p-6 border-b">
            <Link href={`/treasury/${contractAddress}`}>
              <Button variant="outline" size="icon">
                <ArrowLeft className="h-4 w-4" />
                <span className="sr-only">Back</span>
              </Button>
            </Link>
            <div>
              <h1 className="text-2xl font-bold tracking-tight">
                Create a new payout for{" "}
                <span className="text-primary">{treasury?.name}</span>
              </h1>
              <p className="text-muted-foreground">
                Create a new one-time, recurring or vested payout from your
                treasury.
              </p>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto p-6">
            <div className="grid gap-6 lg:grid-cols-3">
              {/* Main Form */}
              <div className="lg:col-span-2">
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <DollarSign className="h-5 w-5 text-primary" />
                      Payment Details
                    </CardTitle>
                    <CardDescription>
                      Fill in the payment information below.
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    {/* Recipient Selection */}
                    <FormField
                      control={control}
                      name="recipient"
                      rules={{ required: "Recipient is required" }}
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel className="flex items-center gap-2">
                            <User className="h-4 w-4 text-accent" />
                            Select a Recipient
                          </FormLabel>
                          <FormControl>
                            <Select
                              value={field.value}
                              onValueChange={field.onChange}
                            >
                              <SelectTrigger className="focus:ring-primary">
                                <SelectValue placeholder="Find or add new recipient" />
                              </SelectTrigger>
                              <SelectContent className="border bg-popover backdrop-blur-md">
                                {recipients.map((recipient) => (
                                  <SelectItem
                                    key={recipient.id}
                                    value={recipient.id}
                                  >
                                    <div className="flex flex-col">
                                      <span className="font-medium">
                                        {recipient.name}
                                      </span>
                                      <span className="text-xs text-muted-foreground truncate max-w-[200px]">
                                        {recipient.address}
                                      </span>
                                    </div>
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                    {/* Currency Labeling */}
                    <FormField
                      control={control}
                      name="labelingCurrency"
                      rules={{ required: "Labeling currency is required" }}
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel className="flex items-center gap-2">
                            Currency (labeling)
                            <div className="h-4 w-4 rounded-full bg-muted-foreground/20 flex items-center justify-center">
                              <span className="text-xs">?</span>
                            </div>
                          </FormLabel>
                          <FormControl>
                            <Select
                              value={field.value}
                              onValueChange={field.onChange}
                            >
                              <SelectTrigger className="focus:ring-primary">
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent className="border bg-popover backdrop-blur-md">
                                {labelingCurrencies.map((currency) => (
                                  <SelectItem
                                    key={currency.id}
                                    value={currency.id}
                                  >
                                    <div className="flex items-center gap-2">
                                      <span>{currency.icon}</span>
                                      <span>{currency.name}</span>
                                    </div>
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                    {/* Payment Amount */}
                    <FormField
                      control={control}
                      name="paymentAmount"
                      rules={{ required: "Payment amount is required" }}
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>Payment Amount</FormLabel>
                          <FormControl>
                            <div className="flex">
                              <Input
                                type="number"
                                placeholder="0"
                                min={0}
                                {...field}
                                className="focus-visible:ring-primary rounded-r-none"
                              />
                              <div className="flex items-center px-3 border border-l-0 bg-muted rounded-r-md">
                                <span className="text-sm text-muted-foreground">
                                  {values.labelingCurrency}
                                </span>
                              </div>
                            </div>
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                    {/* Payment Currency Selection */}
                    <FormField
                      control={control}
                      name="paymentCurrency"
                      rules={{ required: "Payment currency is required" }}
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>
                            Choose the payment currency on Polkadot Asset Hub
                          </FormLabel>
                          <FormControl>
                            <div className="flex flex-wrap gap-2">
                              {paymentCurrencies.map((currency) => (
                                <Button
                                  key={currency.id}
                                  type="button"
                                  variant={
                                    field.value === currency.id
                                      ? "default"
                                      : "outline"
                                  }
                                  size="sm"
                                  onClick={() => field.onChange(currency.id)}
                                >
                                  <span className="mr-2">{currency.icon}</span>
                                  {currency.name}
                                </Button>
                              ))}
                            </div>
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                    {/* Reason */}
                    <FormField
                      control={control}
                      name="reason"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel className="flex items-center gap-2">
                            <FileText className="h-4 w-4 text-primary" />
                            Reason
                          </FormLabel>
                          <FormControl>
                            <Textarea
                              placeholder="Reason (optional)"
                              {...field}
                              className="focus-visible:ring-primary min-h-[80px]"
                            />
                          </FormControl>
                        </FormItem>
                      )}
                    />
                    {/* Tags */}
                    <FormField
                      control={control}
                      name="tags"
                      render={() => (
                        <FormItem>
                          <FormLabel className="flex items-center gap-2">
                            <Tag className="h-4 w-4 text-accent" />
                            Tags
                          </FormLabel>
                          <div className="flex gap-2">
                            <Input
                              placeholder="Select or add a tag"
                              value={newTag}
                              onChange={(e) => setNewTag(e.target.value)}
                              onKeyPress={(e) =>
                                e.key === "Enter" &&
                                (e.preventDefault(), addTag())
                              }
                              className="focus-visible:ring-primary"
                            />
                            <Button
                              type="button"
                              onClick={addTag}
                              variant="outline"
                              size="icon"
                            >
                              <Plus className="h-4 w-4" />
                            </Button>
                          </div>
                          {values.tags.length > 0 && (
                            <div className="flex flex-wrap gap-2">
                              {values.tags.map((tag) => (
                                <Badge
                                  key={tag}
                                  variant="secondary"
                                  className="bg-muted text-foreground border"
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
                        </FormItem>
                      )}
                    />
                    {/* Attached Files */}
                    <FormField
                      control={control}
                      name="attachedFiles"
                      render={() => (
                        <FormItem>
                          <FormLabel>Attached files</FormLabel>
                          {values.attachedFiles.length === 0 ? (
                            <p className="text-sm text-muted-foreground">
                              No file attached yet.
                            </p>
                          ) : (
                            <div className="space-y-2">
                              {values.attachedFiles.map((file, index) => (
                                <div
                                  key={index}
                                  className="flex items-center justify-between p-2 rounded-md bg-muted border"
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
                                className="cursor-pointer"
                                asChild
                              >
                                <span>
                                  <Upload className="mr-2 h-4 w-4" />+ Add a
                                  file
                                </span>
                              </Button>
                            </Label>
                          </div>
                        </FormItem>
                      )}
                    />
                  </CardContent>
                </Card>
              </div>
              {/* Summary Panel */}
              <div className="lg:col-span-1">
                <div className="sticky top-0">
                  <Card className={summaryBg}>
                    <CardHeader>
                      <CardTitle>Direct Payment</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      {/* Treasury */}
                      <div className="flex items-center gap-3">
                        <div
                          className={cn(
                            "w-8 h-8 rounded-full flex items-center justify-center",
                            treasury?.contractAddress
                              ? "bg-green-500/40"
                              : "bg-muted-foreground/20"
                          )}
                        >
                          {treasury?.contractAddress ? (
                            <Check className="h-4 w-4 text-green-500" />
                          ) : (
                            <Check className="h-4 w-4 text-muted-foreground" />
                          )}
                        </div>
                        <div>
                          <p className="text-sm font-medium">
                            Treasury {treasury?.name}
                          </p>
                          <p className="text-xs text-muted-foreground">
                            {trimAddress(treasury?.contractAddress || "", 8)}
                          </p>
                        </div>
                      </div>
                      {/* Recipient */}
                      <div className="flex items-center gap-3">
                        <div
                          className={cn(
                            "w-8 h-8 rounded-full flex items-center justify-center",
                            isRecipientValid
                              ? "bg-green-500/40"
                              : "bg-muted-foreground/20"
                          )}
                        >
                          {errors.recipient ? (
                            <X className="h-4 w-4 text-red-500" />
                          ) : isRecipientValid ? (
                            <Check className="h-4 w-4 text-green-500" />
                          ) : (
                            <Check className="h-4 w-4 text-muted-foreground" />
                          )}
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
                        <div
                          className={cn(
                            "w-8 h-8 rounded-full flex items-center justify-center",
                            isLabelingCurrencyValid
                              ? "bg-green-500/40"
                              : "bg-muted-foreground/20"
                          )}
                        >
                          {errors.labelingCurrency ? (
                            <X className="h-4 w-4 text-red-500" />
                          ) : isLabelingCurrencyValid ? (
                            <Check className="h-4 w-4 text-green-500" />
                          ) : (
                            <Check className="h-4 w-4 text-muted-foreground" />
                          )}
                        </div>
                        <div>
                          <p className="text-sm font-medium">
                            Currency (labeling)
                          </p>
                          <p className="text-xs text-muted-foreground">
                            {values.labelingCurrency || "Not selected"}
                          </p>
                        </div>
                      </div>
                      {/* Payment Amount */}
                      <div className="flex items-center gap-3">
                        <div
                          className={cn(
                            "w-8 h-8 rounded-full flex items-center justify-center",
                            isPaymentAmountValid
                              ? "bg-green-500/40"
                              : "bg-muted-foreground/20"
                          )}
                        >
                          {errors.paymentAmount ? (
                            <X className="h-4 w-4 text-red-500" />
                          ) : isPaymentAmountValid ? (
                            <Check className="h-4 w-4 text-green-500" />
                          ) : (
                            <Check className="h-4 w-4 text-muted-foreground" />
                          )}
                        </div>
                        <div>
                          <p className="text-sm font-medium">Payment Amount</p>
                          <p className="text-xs text-muted-foreground">
                            {values.paymentAmount || "0"}{" "}
                            {values.labelingCurrency}
                          </p>
                        </div>
                      </div>
                      {/* Payment Currency */}
                      <div className="flex items-center gap-3">
                        <div
                          className={cn(
                            "w-8 h-8 rounded-full flex items-center justify-center",
                            isPaymentCurrencyValid
                              ? "bg-green-500/40"
                              : "bg-muted-foreground/20"
                          )}
                        >
                          {errors.paymentCurrency ? (
                            <X className="h-4 w-4 text-red-500" />
                          ) : isPaymentCurrencyValid ? (
                            <Check className="h-4 w-4 text-green-500" />
                          ) : (
                            <Check className="h-4 w-4 text-muted-foreground" />
                          )}
                        </div>
                        <div>
                          <p className="text-sm font-medium">
                            Payment Currency
                          </p>
                          <p className="text-xs text-muted-foreground">
                            {selectedPaymentCurrency?.name
                              ? `${selectedPaymentCurrency.name} on Polkadot Asset Hub`
                              : "Not selected"}
                          </p>
                        </div>
                      </div>
                      {/* Error Display */}
                      {addPayoutError && (
                        <div className="rounded-md border border-red-500/20 bg-red-950/20 p-3">
                          <p className="text-sm text-red-400">
                            Error: {addPayoutError.message}
                          </p>
                        </div>
                      )}

                      {/* Action Buttons */}
                      <div className="pt-4 space-y-2">
                        <Button
                          type="submit"
                          className="w-full"
                          disabled={!isComplete || isAddingPayout}
                        >
                          {isAddingPayout && (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          )}
                          {isAddingPayout ? "Adding..." : "Pay with next batch"}
                        </Button>
                        <Button
                          type="button"
                          onClick={handleSubmit((data) => onSubmit(data, true))}
                          variant="outline"
                          className="w-full"
                          disabled={!isComplete || isAddingPayout}
                        >
                          {isAddingPayout && (
                            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          )}
                          {isAddingPayout ? "Adding..." : "Pay Now"}
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </div>
            </div>
          </div>
        </div>
      </form>
    </Form>
  );
}

"use client";

import type React from "react";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import {
  ArrowLeft,
  Check,
  Sparkles,
  Wallet,
  CreditCard,
  Calendar,
  Globe,
  User,
  X,
  Loader2,
} from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";
import { useForm } from "react-hook-form";
import {
  Form,
  FormField,
  FormItem,
  FormLabel,
  FormControl,
  FormMessage,
} from "@/components/ui/form";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import { usePolkadotExtension } from "@/providers/polkadot-extension-provider";
import { useDeployTreasury } from "@/hooks/use-deploy-treasury";
import { CONTRACT_NETWORKS, NetworkId } from "@/lib/treasury-contract-service";

interface Treasurer {
  name: string;
  address: string;
}

interface CreateTreasuryFormValues {
  name: string;
  description: string;
  currencies: string[];
  payoutFrequency: string;
  treasurers: Treasurer[];
  network: NetworkId;
}

export function CreateTreasuryForm() {
  const router = useRouter();
  const { selectedAccount } = usePolkadotExtension();
  const {
    deployTreasury,
    contractAddress,
    isLoading: isDeploying,
    error: deployError,
    isSuccess: deploySuccess,
    reset: resetDeploy,
  } = useDeployTreasury();
  const [step, setStep] = useState(1);
  const [treasurerInput, setTreasurerInput] = useState("");
  const [treasurerError, setTreasurerError] = useState("");

  const form = useForm<CreateTreasuryFormValues>({
    defaultValues: {
      name: "",
      description: "",
      currencies: ["DOT"],
      payoutFrequency: "monthly",
      network: "POP_NETWORK",
      treasurers: [
        {
          name: "Yourself",
          address: selectedAccount?.address || "",
        },
      ],
    },
  });

  useEffect(() => {
    if (selectedAccount) {
      setValue("treasurers", [
        { name: "Yourself", address: selectedAccount.address },
      ]);
    }
  }, [selectedAccount]);

  const { control, handleSubmit, watch, setValue } = form;
  const values = watch();

  const isValidSS58 = (address: string) => {
    return /^5[0-9a-zA-Z]{47,48}$/.test(address);
  };

  const addTreasurer = () => {
    if (!treasurerInput.trim()) {
      setTreasurerError("Please enter an address");
      return;
    }

    if (!isValidSS58(treasurerInput)) {
      setTreasurerError("Please enter a valid ss58 address");
      return;
    }

    if (values.treasurers.some((t) => t.address === treasurerInput)) {
      setTreasurerError("This address is already added");
      return;
    }

    setValue("treasurers", [
      ...values.treasurers,
      {
        name: `Treasurer ${values.treasurers.length}`,
        address: treasurerInput,
      },
    ]);

    setTreasurerInput("");
    setTreasurerError("");
  };

  const removeTreasurer = (address: string) => {
    setValue(
      "treasurers",
      values.treasurers.filter((t) => t.address !== address)
    );
  };

  const handleCurrencyChange = (currency: string, checked: boolean) => {
    setValue(
      "currencies",
      checked
        ? [...values.currencies, currency]
        : values.currencies.filter((c) => c !== currency)
    );
  };

  function onSubmit(data: CreateTreasuryFormValues) {
    console.log("Form submitted:", data);
    deployTreasury(data);
  }

  // Handle successful deployment
  useEffect(() => {
    if (deploySuccess && contractAddress) {
      console.log("Treasury deployed with address:", contractAddress);
      // TODO: Add a success message and redirect to the treasury page
      // router.push(`/treasury/${encodeURIComponent(contractAddress)}`);
    }
  }, [deploySuccess, contractAddress, router]);

  const nextStep = () => setStep(step + 1);
  const prevStep = () => setStep(step - 1);

  return (
    <>
      <div className="flex gap-4 items-center">
        <Link href="/dashboard">
          <Button variant="outline" size="icon" className="cursor-pointer">
            <ArrowLeft className="w-4 h-4" />
            <span className="sr-only">Back</span>
          </Button>
        </Link>
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Create Treasury</h1>
          <p className="text-muted-foreground">
            Set up a new treasury for your organization.
          </p>
        </div>
      </div>
      <Card>
        <CardHeader>
          <div className="flex gap-2 items-center">
            <Sparkles className="w-5 h-5 text-primary animate-pulse-glow" />
            <CardTitle>Treasury Details</CardTitle>
          </div>
          <CardDescription>
            Fill in the details to create your new treasury.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
              {step === 1 && (
                <div className="grid gap-6">
                  <FormField
                    control={control}
                    name="name"
                    rules={{ required: "Treasury name is required" }}
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel className="flex gap-2 items-center">
                          <Wallet className="w-4 h-4 text-primary" />
                          Treasury Name
                        </FormLabel>
                        <FormControl>
                          <Input
                            placeholder="e.g. Operations Fund"
                            {...field}
                            className="focus-visible:ring-primary"
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={control}
                    name="description"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel className="flex gap-2 items-center">
                          <CreditCard className="w-4 h-4 text-accent" />
                          Description
                        </FormLabel>
                        <FormControl>
                          <Textarea
                            placeholder="Describe the purpose of this treasury"
                            {...field}
                            className="min-h-[100px] focus-visible:ring-primary"
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <div className="grid gap-3">
                    <Label className="flex gap-2 items-center">
                      <User className="w-4 h-4 text-primary" />
                      Treasurers
                    </Label>
                    <div className="space-y-4">
                      <div className="flex gap-2">
                        <div className="flex-1">
                          <Input
                            placeholder="Enter ss58 address"
                            value={treasurerInput}
                            onChange={(e) => setTreasurerInput(e.target.value)}
                            className="focus-visible:ring-primary"
                          />
                          {treasurerError && (
                            <p className="mt-1 text-xs text-red-400">
                              {treasurerError}
                            </p>
                          )}
                        </div>
                        <Button
                          type="button"
                          onClick={addTreasurer}
                          variant="outline"
                        >
                          Add
                        </Button>
                      </div>

                      <div className="p-2 rounded-md border bg-muted/20">
                        <div className="mb-2 text-sm font-medium">
                          Treasury Managers
                        </div>
                        {values.treasurers.length === 0 ? (
                          <p className="text-sm text-muted-foreground">
                            No treasurers added yet
                          </p>
                        ) : (
                          <div className="space-y-2">
                            {values.treasurers.map((treasurer, index) => (
                              <div
                                key={treasurer.address}
                                className="flex justify-between items-center p-2 rounded-md bg-muted/50"
                              >
                                <div className="flex flex-col">
                                  <span className="text-sm font-medium">
                                    {treasurer.name}
                                  </span>
                                  <span className="text-xs text-muted-foreground truncate max-w-[300px]">
                                    {treasurer.address}
                                  </span>
                                </div>
                                {index !== 0 && (
                                  <Button
                                    type="button"
                                    variant="ghost"
                                    size="sm"
                                    onClick={() =>
                                      removeTreasurer(treasurer.address)
                                    }
                                    className="p-0 w-8 h-8 text-muted-foreground hover:text-destructive"
                                  >
                                    <X className="w-4 h-4" />
                                    <span className="sr-only">Remove</span>
                                  </Button>
                                )}
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>

                  <FormField
                    control={control}
                    name="network"
                    render={({ field }) => (
                      <FormItem className="space-y-3">
                        <FormLabel className="flex gap-2 items-center">
                          <Globe className="w-4 h-4 text-primary" />
                          Deployment Network
                        </FormLabel>
                        <FormControl>
                          <RadioGroup
                            onValueChange={field.onChange}
                            defaultValue={field.value}
                            className="grid grid-cols-1 gap-4 sm:grid-cols-2"
                          >
                            {Object.entries(CONTRACT_NETWORKS).map(
                              ([key, network]) => (
                                <div key={key}>
                                  <RadioGroupItem
                                    value={key}
                                    id={network.id}
                                    className="sr-only peer"
                                  />
                                  <Label
                                    htmlFor={network.id}
                                    className="flex flex-col items-start justify-between rounded-md border bg-muted/20 p-4 hover:bg-muted/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary cursor-pointer"
                                  >
                                    <span className="mb-1 text-base font-medium">
                                      {network.name}
                                    </span>
                                    <span className="font-mono text-xs text-muted-foreground">
                                      {network.endpoint}
                                    </span>
                                  </Label>
                                </div>
                              )
                            )}
                          </RadioGroup>
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <div className="grid gap-3">
                    <Label className="flex gap-2 items-center">
                      <Globe className="w-4 h-4 text-accent" />
                      Currencies
                    </Label>
                    <div className="grid gap-4">
                      <div className="flex items-center space-x-2">
                        <Checkbox
                          id="currency-dot"
                          checked={values.currencies.includes("DOT")}
                          onCheckedChange={(checked) =>
                            handleCurrencyChange("DOT", checked as boolean)
                          }
                          className="data-[state=checked]:bg-primary data-[state=checked]:border-primary"
                        />
                        <Label htmlFor="currency-dot" className="font-medium">
                          DOT
                        </Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Checkbox id="currency-usdc" disabled />
                        <Label
                          htmlFor="currency-usdc"
                          className="text-muted-foreground"
                        >
                          USDC{" "}
                          <span className="ml-2 text-xs opacity-70">
                            Coming soon
                          </span>
                        </Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Checkbox id="currency-usdt" disabled />
                        <Label
                          htmlFor="currency-usdt"
                          className="text-muted-foreground"
                        >
                          USDT{" "}
                          <span className="ml-2 text-xs opacity-70">
                            Coming soon
                          </span>
                        </Label>
                      </div>
                    </div>
                  </div>

                  <FormField
                    control={control}
                    name="payoutFrequency"
                    render={({ field }) => (
                      <FormItem className="space-y-3">
                        <FormLabel className="flex gap-2 items-center">
                          <Calendar className="w-4 h-4 text-primary" />
                          Payout Frequency
                        </FormLabel>
                        <FormControl>
                          <RadioGroup
                            onValueChange={field.onChange}
                            defaultValue={field.value}
                            className="grid grid-cols-2 gap-4 sm:grid-cols-4"
                          >
                            <div>
                              <RadioGroupItem
                                value="weekly"
                                id="weekly"
                                className="sr-only peer"
                              />
                              <Label
                                htmlFor="weekly"
                                className="flex flex-col items-center justify-between rounded-md border bg-muted/20 p-4 hover:bg-muted/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                              >
                                <span className="mb-1 font-medium">Weekly</span>
                              </Label>
                            </div>
                            <div>
                              <RadioGroupItem
                                value="biweekly"
                                id="biweekly"
                                className="sr-only peer"
                              />
                              <Label
                                htmlFor="biweekly"
                                className="flex flex-col items-center justify-between rounded-md border bg-muted/20 p-4 hover:bg-muted/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                              >
                                <span className="mb-1 font-medium">
                                  Bi-weekly
                                </span>
                              </Label>
                            </div>
                            <div>
                              <RadioGroupItem
                                value="monthly"
                                id="monthly"
                                className="sr-only peer"
                              />
                              <Label
                                htmlFor="monthly"
                                className="flex flex-col items-center justify-between rounded-md border bg-muted/20 p-4 hover:bg-muted/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                              >
                                <span className="mb-1 font-medium">
                                  Monthly
                                </span>
                              </Label>
                            </div>
                            <div>
                              <RadioGroupItem
                                value="quarterly"
                                id="quarterly"
                                className="sr-only peer"
                              />
                              <Label
                                htmlFor="quarterly"
                                className="flex flex-col items-center justify-between rounded-md border bg-muted/20 p-4 hover:bg-muted/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                              >
                                <span className="mb-1 font-medium">
                                  Quarterly
                                </span>
                              </Label>
                            </div>
                          </RadioGroup>
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
              )}

              {step === 2 && (
                <div className="space-y-6">
                  <div className="p-6 rounded-lg border backdrop-blur-md bg-muted/20">
                    <h3 className="flex gap-2 items-center mb-4 text-lg font-medium">
                      <Sparkles className="w-5 h-5 text-primary" />
                      Treasury Summary
                    </h3>
                    <div className="grid gap-4">
                      <div className="grid grid-cols-2 gap-2">
                        <div className="text-sm font-medium text-muted-foreground">
                          Name
                        </div>
                        <div className="text-sm">{values.name}</div>
                      </div>
                      <Separator />
                      <div className="grid grid-cols-2 gap-2">
                        <div className="text-sm font-medium text-muted-foreground">
                          Description
                        </div>
                        <div className="text-sm">
                          {values.description || "N/A"}
                        </div>
                      </div>
                      <Separator />
                      <div className="grid grid-cols-2 gap-2">
                        <div className="text-sm font-medium text-muted-foreground">
                          Currencies
                        </div>
                        <div className="text-sm">
                          {values.currencies.join(", ")}
                        </div>
                      </div>
                      <Separator />
                      <div className="grid grid-cols-2 gap-2">
                        <div className="text-sm font-medium text-muted-foreground">
                          Payout Frequency
                        </div>
                        <div className="text-sm capitalize">
                          {values.payoutFrequency}
                        </div>
                      </div>
                      <Separator />
                      <div className="grid grid-cols-2 gap-2">
                        <div className="text-sm font-medium text-muted-foreground">
                          Network
                        </div>
                        <div className="text-sm">
                          {CONTRACT_NETWORKS[values.network]?.name}
                        </div>
                      </div>
                      <Separator />
                      <div className="grid grid-cols-2 gap-2">
                        <div className="text-sm font-medium text-muted-foreground">
                          Treasurers
                        </div>
                        <div className="text-sm">
                          {values.treasurers.map((t) => t.name).join(", ")}
                        </div>
                      </div>
                    </div>
                  </div>
                  <div className="p-4 rounded-lg border backdrop-blur-md border-primary/20 bg-primary/10">
                    <div className="flex gap-2 items-center text-primary">
                      <Check className="w-5 h-5" />
                      <p className="text-sm font-medium">
                        Your treasury is ready to be created
                      </p>
                    </div>
                  </div>

                  {deployError && (
                    <div className="p-4 rounded-lg border backdrop-blur-md border-destructive/20 bg-destructive/10">
                      <div className="flex justify-between items-center">
                        <div className="flex gap-2 items-center text-destructive">
                          <X className="w-5 h-5" />
                          <div>
                            <p className="text-sm font-medium">
                              Deployment Error
                            </p>
                            <p className="mt-1 text-xs text-destructive/80">
                              {deployError}
                            </p>
                          </div>
                        </div>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={resetDeploy}
                          className="border-destructive/20 bg-destructive/10 hover:bg-destructive/20 text-destructive"
                        >
                          Try Again
                        </Button>
                      </div>
                    </div>
                  )}
                </div>
              )}

              <div className="flex justify-between mt-6">
                {step > 1 ? (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={prevStep}
                    disabled={isDeploying}
                  >
                    Previous
                  </Button>
                ) : (
                  <div></div>
                )}
                {step < 2 ? (
                  <Button type="button" onClick={nextStep}>
                    Review Details
                  </Button>
                ) : (
                  <Button type="submit" disabled={isDeploying}>
                    {isDeploying && (
                      <Loader2 className="mr-2 w-4 h-4 animate-spin" />
                    )}
                    {isDeploying ? "Deploying Treasury..." : "Create Treasury"}
                  </Button>
                )}
              </div>
            </form>
          </Form>
        </CardContent>
        <CardFooter className="px-6 py-4 border-t bg-muted/20">
          <p className="text-xs text-muted-foreground">
            By creating a treasury, you agree to our Terms of Service and
            Privacy Policy.
          </p>
        </CardFooter>
      </Card>
    </>
  );
}

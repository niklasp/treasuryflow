"use client";

import type React from "react";

import { useState } from "react";
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
} from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox";

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

export function CreateTreasuryForm() {
  const router = useRouter();
  const [step, setStep] = useState(1);
  // Update the formData state to include treasurers
  const [formData, setFormData] = useState({
    name: "",
    description: "",
    currencies: ["DOT"],
    payoutFrequency: "monthly",
    treasurers: [
      {
        name: "Yourself",
        address: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
      },
    ],
  });

  // Add a new state for the treasurer input
  const [treasurerInput, setTreasurerInput] = useState("");
  const [treasurerError, setTreasurerError] = useState("");

  // Add a function to validate ss58 addresses (basic check)
  const isValidSS58 = (address: string) => {
    // Basic validation - ss58 addresses start with "5" and are typically 48 characters
    return /^5[0-9a-zA-Z]{47,48}$/.test(address);
  };

  // Add a function to add a treasurer
  const addTreasurer = () => {
    if (!treasurerInput.trim()) {
      setTreasurerError("Please enter an address");
      return;
    }

    if (!isValidSS58(treasurerInput)) {
      setTreasurerError("Please enter a valid ss58 address");
      return;
    }

    // Check if address already exists
    if (formData.treasurers.some((t) => t.address === treasurerInput)) {
      setTreasurerError("This address is already added");
      return;
    }

    setFormData((prev) => ({
      ...prev,
      treasurers: [
        ...prev.treasurers,
        {
          name: `Treasurer ${prev.treasurers.length}`,
          address: treasurerInput,
        },
      ],
    }));

    setTreasurerInput("");
    setTreasurerError("");
  };

  // Add a function to remove a treasurer
  const removeTreasurer = (address: string) => {
    setFormData((prev) => ({
      ...prev,
      treasurers: prev.treasurers.filter((t) => t.address !== address),
    }));
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const handleSelectChange = (name: string, value: string) => {
    setFormData((prev) => ({ ...prev, [name]: value }));
  };

  const handleCurrencyChange = (currency: string, checked: boolean) => {
    setFormData((prev) => ({
      ...prev,
      currencies: checked
        ? [...prev.currencies, currency]
        : prev.currencies.filter((c) => c !== currency),
    }));
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // In a real app, you would submit the form data to your backend
    console.log("Form submitted:", formData);
    // Redirect to dashboard
    router.push("/dashboard");
  };

  const nextStep = () => setStep(step + 1);
  const prevStep = () => setStep(step - 1);

  return (
    <>
      <div className="flex items-center gap-4">
        <Link href="/dashboard">
          <Button
            variant="outline"
            size="icon"
            className="border-white/5 bg-black/20 hover:bg-black/40 cursor-pointer"
          >
            <ArrowLeft className="h-4 w-4" />
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
      <Card className="border-white/5 bg-black/40 backdrop-blur-md overflow-hidden">
        <CardHeader>
          <div className="flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-purple-500 animate-pulse-glow" />
            <CardTitle>Treasury Details</CardTitle>
          </div>
          <CardDescription>
            Fill in the details to create your new treasury.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit}>
            {step === 1 && (
              <div className="grid gap-6">
                <div className="grid gap-3">
                  <Label htmlFor="name" className="flex items-center gap-2">
                    <Wallet className="h-4 w-4 text-purple-500" />
                    Treasury Name
                  </Label>
                  <Input
                    id="name"
                    name="name"
                    placeholder="e.g. Operations Fund"
                    value={formData.name}
                    onChange={handleChange}
                    required
                    className="border-white/5 bg-black/20 focus-visible:ring-primary"
                  />
                </div>
                <div className="grid gap-3">
                  <Label
                    htmlFor="description"
                    className="flex items-center gap-2"
                  >
                    <CreditCard className="h-4 w-4 text-blue-500" />
                    Description
                  </Label>
                  <Textarea
                    id="description"
                    name="description"
                    placeholder="Describe the purpose of this treasury"
                    value={formData.description}
                    onChange={handleChange}
                    className="min-h-[100px] border-white/5 bg-black/20 focus-visible:ring-primary"
                  />
                </div>
                <div className="grid gap-3">
                  <Label className="flex items-center gap-2">
                    <User className="h-4 w-4 text-yellow-500" />
                    Treasurers
                  </Label>
                  <div className="space-y-4">
                    <div className="flex gap-2">
                      <div className="flex-1">
                        <Input
                          placeholder="Enter ss58 address"
                          value={treasurerInput}
                          onChange={(e) => setTreasurerInput(e.target.value)}
                          className="border-white/5 bg-black/20 focus-visible:ring-primary"
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
                        className="border-white/5 bg-black/20 hover:bg-black/40"
                      >
                        Add
                      </Button>
                    </div>

                    <div className="rounded-md border border-white/5 bg-black/20 p-2">
                      <div className="text-sm font-medium mb-2">
                        Treasury Managers
                      </div>
                      {formData.treasurers.length === 0 ? (
                        <p className="text-sm text-muted-foreground">
                          No treasurers added yet
                        </p>
                      ) : (
                        <div className="space-y-2">
                          {formData.treasurers.map((treasurer, index) => (
                            <div
                              key={treasurer.address}
                              className="flex items-center justify-between rounded-md bg-black/30 p-2"
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
                                  className="h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
                                >
                                  <X className="h-4 w-4" />
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
                <div className="grid gap-3">
                  <Label className="flex items-center gap-2">
                    <Globe className="h-4 w-4 text-green-500" />
                    Currencies
                  </Label>
                  <div className="grid gap-4">
                    <div className="flex items-center space-x-2">
                      <Checkbox
                        id="currency-dot"
                        checked={formData.currencies.includes("DOT")}
                        onCheckedChange={(checked) =>
                          handleCurrencyChange("DOT", checked as boolean)
                        }
                        className="border-white/20 data-[state=checked]:bg-primary data-[state=checked]:border-primary"
                      />
                      <Label htmlFor="currency-dot" className="font-medium">
                        DOT
                      </Label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Checkbox
                        id="currency-usdc"
                        disabled
                        className="border-white/20"
                      />
                      <Label
                        htmlFor="currency-usdc"
                        className="text-muted-foreground"
                      >
                        USDC{" "}
                        <span className="text-xs ml-2 opacity-70">
                          Coming soon
                        </span>
                      </Label>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Checkbox
                        id="currency-usdt"
                        disabled
                        className="border-white/20"
                      />
                      <Label
                        htmlFor="currency-usdt"
                        className="text-muted-foreground"
                      >
                        USDT{" "}
                        <span className="text-xs ml-2 opacity-70">
                          Coming soon
                        </span>
                      </Label>
                    </div>
                  </div>
                </div>
                <div className="grid gap-3">
                  <Label
                    htmlFor="payoutFrequency"
                    className="flex items-center gap-2"
                  >
                    <Calendar className="h-4 w-4 text-rose-500" />
                    Payout Frequency
                  </Label>
                  <RadioGroup
                    defaultValue={formData.payoutFrequency}
                    onValueChange={(value) =>
                      handleSelectChange("payoutFrequency", value)
                    }
                    className="grid grid-cols-2 gap-4 sm:grid-cols-4"
                  >
                    <div>
                      <RadioGroupItem
                        value="weekly"
                        id="weekly"
                        className="peer sr-only"
                      />
                      <Label
                        htmlFor="weekly"
                        className="flex flex-col items-center justify-between rounded-md border border-white/5 bg-black/20 p-4 hover:bg-black/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                      >
                        <span className="mb-1 font-medium">Weekly</span>
                      </Label>
                    </div>
                    <div>
                      <RadioGroupItem
                        value="biweekly"
                        id="biweekly"
                        className="peer sr-only"
                      />
                      <Label
                        htmlFor="biweekly"
                        className="flex flex-col items-center justify-between rounded-md border border-white/5 bg-black/20 p-4 hover:bg-black/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                      >
                        <span className="mb-1 font-medium">Bi-weekly</span>
                      </Label>
                    </div>
                    <div>
                      <RadioGroupItem
                        value="monthly"
                        id="monthly"
                        className="peer sr-only"
                      />
                      <Label
                        htmlFor="monthly"
                        className="flex flex-col items-center justify-between rounded-md border border-white/5 bg-black/20 p-4 hover:bg-black/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                      >
                        <span className="mb-1 font-medium">Monthly</span>
                      </Label>
                    </div>
                    <div>
                      <RadioGroupItem
                        value="quarterly"
                        id="quarterly"
                        className="peer sr-only"
                      />
                      <Label
                        htmlFor="quarterly"
                        className="flex flex-col items-center justify-between rounded-md border border-white/5 bg-black/20 p-4 hover:bg-black/40 hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary"
                      >
                        <span className="mb-1 font-medium">Quarterly</span>
                      </Label>
                    </div>
                  </RadioGroup>
                </div>
              </div>
            )}
            {step === 2 && (
              <div className="space-y-6">
                <div className="rounded-lg border border-white/5 bg-black/20 backdrop-blur-md p-6">
                  <h3 className="mb-4 text-lg font-medium flex items-center gap-2">
                    <Sparkles className="h-5 w-5 text-purple-500" />
                    Treasury Summary
                  </h3>
                  <div className="grid gap-4">
                    <div className="grid grid-cols-2 gap-2">
                      <div className="text-sm font-medium text-muted-foreground">
                        Name
                      </div>
                      <div className="text-sm">{formData.name}</div>
                    </div>
                    <Separator className="bg-white/5" />
                    <div className="grid grid-cols-2 gap-2">
                      <div className="text-sm font-medium text-muted-foreground">
                        Description
                      </div>
                      <div className="text-sm">
                        {formData.description || "N/A"}
                      </div>
                    </div>
                    <Separator className="bg-white/5" />
                    <div className="grid grid-cols-2 gap-2">
                      <div className="text-sm font-medium text-muted-foreground">
                        Currencies
                      </div>
                      <div className="text-sm">
                        {formData.currencies.join(", ")}
                      </div>
                    </div>
                    <Separator className="bg-white/5" />
                    <div className="grid grid-cols-2 gap-2">
                      <div className="text-sm font-medium text-muted-foreground">
                        Payout Frequency
                      </div>
                      <div className="text-sm capitalize">
                        {formData.payoutFrequency}
                      </div>
                    </div>
                    {/* Update the step 2 summary to include treasurers */}
                    {/* Add this after the payout frequency section in step 2 */}
                    <Separator className="bg-white/5" />
                    <div className="grid grid-cols-2 gap-2">
                      <div className="text-sm font-medium text-muted-foreground">
                        Treasurers
                      </div>
                      <div className="text-sm">
                        {formData.treasurers.map((t) => t.name).join(", ")}
                      </div>
                    </div>
                  </div>
                </div>
                <div className="rounded-lg border border-green-500/20 bg-green-950/20 backdrop-blur-md p-4">
                  <div className="flex items-center gap-2 text-green-400">
                    <Check className="h-5 w-5" />
                    <p className="text-sm font-medium">
                      Your treasury is ready to be created
                    </p>
                  </div>
                </div>
              </div>
            )}
            <div className="mt-6 flex justify-between">
              {step > 1 ? (
                <Button
                  type="button"
                  variant="outline"
                  onClick={prevStep}
                  className="border-white/5 bg-black/20 hover:bg-black/40"
                >
                  Previous
                </Button>
              ) : (
                <div></div>
              )}
              {step < 2 ? (
                <Button
                  type="button"
                  onClick={nextStep}
                  className="primary-gradient hover:primary-gradient-hover glow"
                >
                  Review Details
                </Button>
              ) : (
                <Button
                  type="submit"
                  className="primary-gradient hover:primary-gradient-hover glow"
                >
                  Create Treasury
                </Button>
              )}
            </div>
          </form>
        </CardContent>
        <CardFooter className="border-t border-white/5 bg-black/20 px-6 py-4">
          <p className="text-xs text-muted-foreground">
            By creating a treasury, you agree to our Terms of Service and
            Privacy Policy.
          </p>
        </CardFooter>
      </Card>
    </>
  );
}

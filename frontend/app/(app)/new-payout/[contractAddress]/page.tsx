"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { HexString } from "polkadot-api";
import { AddPayoutForm } from "@/components/add-payout-form";

export default function NewPayoutPage() {
  const params = useParams();
  const [contractAddress, setContractAddress] = useState<string>("");

  useEffect(() => {
    if (params?.contractAddress) {
      setContractAddress(decodeURIComponent(params.contractAddress as string));
    }
  }, [params]);

  if (!contractAddress) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <h2 className="text-xl font-semibold mb-2">Loading...</h2>
          <p className="text-muted-foreground">Preparing payout form</p>
        </div>
      </div>
    );
  }

  return <AddPayoutForm contractAddress={contractAddress as HexString} />;
}

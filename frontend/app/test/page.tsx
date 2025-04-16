"use client";

import { useEffect } from "react";
import { contracts } from "@polkadot-api/descriptors";

export default function Test() {
  useEffect(() => {
    console.log(contracts.contract_payouts);
  }, []);

  return <div>test</div>;
}

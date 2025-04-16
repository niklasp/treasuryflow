"use client";

import { useRpcApi } from "@/providers/rpc-api-provider";
import { WsEvent } from "polkadot-api/ws-provider/web";
import type { Subscription } from "rxjs";
import { useEffect, useRef, useState } from "react";

export function useBlockNumber() {
  const { client, connectionStatus } = useRpcApi();
  const [blockNumber, setBlockNumber] = useState<number | null>(null);
  const subscription = useRef<Subscription | null>(null);

  useEffect(() => {
    if (client && connectionStatus?.type === WsEvent.CONNECTED) {
      subscription.current = client.finalizedBlock$.subscribe((value) => {
        setBlockNumber(value.number);
      });
    }

    return () => {
      subscription.current?.unsubscribe();
      subscription.current = null;
      setBlockNumber(null);
    };
  }, [client, connectionStatus?.type]);

  return blockNumber;
}

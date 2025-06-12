"use client";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useBlockNumber } from "@/hooks/use-block-number";
import { WsEvent } from "polkadot-api/ws-provider/web";
import { useMemo, useState } from "react";
import { useLightClientApi } from "@/providers/lightclient-api-provider";

export function ChainInfo() {
  const { connectionStatus, activeChain } = useLightClientApi();
  const [isOpen, setIsOpen] = useState(false);
  const blockNumber = useBlockNumber();

  const status: "connected" | "error" | "connecting" =
    connectionStatus?.type === WsEvent.CONNECTED && blockNumber
      ? "connected"
      : connectionStatus?.type === WsEvent.ERROR ||
          connectionStatus?.type === WsEvent.CLOSE
        ? "error"
        : "connecting";

  const Trigger = useMemo(() => {
    return (
      <div className="tabular-nums font-light h-6 border-foreground/20 border rounded-md px-2 text-[12px] cursor-default">
        {status === "connected" ? (
          <>
            <span className="block rounded-full w-2 h-2 bg-green-400 animate-pulse mr-1" />
          </>
        ) : status === "error" ? (
          <>
            <span className="block rounded-full w-2.5 h-2.5 bg-red-400" />
          </>
        ) : (
          <>
            <span className="block rounded-full w-2 h-2 bg-yellow-400 animate-pulse" />
            &nbsp; connecting to {activeChain?.name} via lightclient
          </>
        )}
        {status === "connected" && blockNumber && (
          <span className="text-[10px]">{`#${blockNumber}`}</span>
        )}
      </div>
    );
  }, [status, blockNumber, activeChain]);

  const Content = useMemo(() => {
    return (
      <>
        {status === "connected" ? (
          <>
            connected to <b>{activeChain?.name}</b> via lightclient
          </>
        ) : status === "error" ? (
          <>
            error:{" "}
            {connectionStatus?.type === WsEvent.ERROR
              ? "Connection error"
              : "Connection closed"}
          </>
        ) : (
          <>connecting to {activeChain?.name} via lightclient</>
        )}
      </>
    );
  }, [status, activeChain, connectionStatus]);

  return (
    <TooltipProvider>
      <Tooltip delayDuration={100} open={isOpen} onOpenChange={setIsOpen}>
        <TooltipTrigger
          asChild
          className="flex items-center fixed bottom-4 right-4"
        >
          {Trigger}
        </TooltipTrigger>
        <TooltipContent side="right" className="">
          {Content}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

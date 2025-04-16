"use client";

import { createClient, PolkadotClient } from "polkadot-api";
import {
  getWsProvider,
  StatusChange,
  WsJsonRpcProvider,
} from "polkadot-api/ws-provider/web";
import { withPolkadotSdkCompat } from "polkadot-api/polkadot-sdk-compat";

import { createContext, useContext, useEffect, useRef, useState } from "react";
import {
  chainConfig,
  type ChainConfig,
  type AvailableApis,
} from "@/papi-config";

interface RpcApiProviderType {
  connectionStatus: StatusChange | undefined;
  activeChain: ChainConfig | null;
  setActiveChain: (chain: ChainConfig) => void;
  client: PolkadotClient | null;
  wsProvider: WsJsonRpcProvider | null;
  api: AvailableApis | null;
  retryCount: number;
  errorCount: number;
}

const RpcApiContext = createContext<RpcApiProviderType | undefined>(undefined);

const MAX_RETRIES = 3;
const RETRY_INTERVAL = 5000;
const MAX_ERRORS = 5;

export function RpcApiProvider({
  children,
  defaultChain = chainConfig[0],
}: {
  children: React.ReactNode;
  defaultChain?: ChainConfig;
}) {
  const wsProviderRef = useRef<WsJsonRpcProvider | null>(null);
  const [activeChain, _setActiveChain] = useState<ChainConfig>(defaultChain);
  const [activeApi, setActiveApi] = useState<AvailableApis | null>(null);
  const clientRef = useRef<PolkadotClient | null>(null);
  const [retryCount, setRetryCount] = useState(0);
  const [errorCount, setErrorCount] = useState(0);
  const retryTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<
    StatusChange | undefined
  >(undefined);

  const initializeClient = async (chain: ChainConfig) => {
    if (errorCount >= MAX_ERRORS) {
      console.error(
        "Maximum error count reached. Stopping connection attempts.",
      );
      return;
    }

    try {
      const wsEndpoint = handleWsEndpoint({
        defaultEndpoint: chain.endpoints[0],
      });
      if (!wsEndpoint) throw new Error("No valid WebSocket endpoint found");

      const endpoints = [wsEndpoint, ...chain.endpoints.slice(1)];
      const _wsProvider = getWsProvider(endpoints, (status: StatusChange) => {
        setConnectionStatus(status);
        if (!_wsProvider || retryCount < MAX_RETRIES) {
          setRetryCount((prev) => prev + 1);
          retryTimeoutRef.current = setTimeout(
            () => initializeClient(chain),
            RETRY_INTERVAL,
          );
        }
      });

      wsProviderRef.current = _wsProvider;

      const client = createClient(withPolkadotSdkCompat(_wsProvider));
      const api = client.getTypedApi(chain.descriptors);

      clientRef.current = client;
      setActiveApi(api);
      setRetryCount(0); // Reset retry count on successful connection
    } catch (error) {
      console.error("Error connecting to chain", error);
      setErrorCount((prev) => prev + 1);
      if (retryCount < MAX_RETRIES && errorCount < MAX_ERRORS) {
        setRetryCount((prev) => prev + 1);
        retryTimeoutRef.current = setTimeout(
          () => initializeClient(chain),
          RETRY_INTERVAL,
        );
      }
    }
  };

  const setActiveChain = (newChain: ChainConfig) => {
    _setActiveChain(newChain);
    setRetryCount(0); // Reset retry count when changing chains
    setErrorCount(0); // Reset error count when changing chains
    initializeClient(newChain);
  };

  useEffect(() => {
    initializeClient(defaultChain);
    return () => {
      if (retryTimeoutRef.current) {
        clearTimeout(retryTimeoutRef.current);
      }
    };
  }, [defaultChain]);

  return (
    <RpcApiContext.Provider
      value={{
        connectionStatus,
        api: activeApi,
        wsProvider: wsProviderRef.current,
        client: clientRef.current,
        activeChain,
        setActiveChain,
        retryCount,
        errorCount,
      }}
    >
      {children}
    </RpcApiContext.Provider>
  );
}

export function useRpcApi() {
  const context = useContext(RpcApiContext);
  if (!context) {
    throw new Error("useRpcApi must be used within a RpcApiProvider");
  }
  return context;
}

/**
 * Get or set the WebSocket endpoint from URL search params
 * Default endpoint will be used if none is specified
 */
export function handleWsEndpoint({
  defaultEndpoint,
}: {
  defaultEndpoint?: string;
} = {}) {
  if (typeof window === "undefined") return defaultEndpoint;

  const params = new URLSearchParams(window.location.search);
  const wsEndpoint = params.get("rpc");

  if (!wsEndpoint) return defaultEndpoint;

  // Validate endpoint is a valid WSS URL
  try {
    const url = new URL(wsEndpoint);
    if (url.protocol !== "wss:") return defaultEndpoint;
    return wsEndpoint;
  } catch {
    return defaultEndpoint;
  }
}

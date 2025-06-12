"use client";

import {
  dot,
  polkadot_asset_hub,
  paseo,
  paseo_asset_hub,
} from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import { logos } from "@/icons/logos";
import { chainSpec as polkadotChainSpec } from "polkadot-api/chains/polkadot";
import { chainSpec as polkadotAssetHubChainSpec } from "polkadot-api/chains/polkadot_asset_hub";
import { chainSpec as paseoChainSpec } from "polkadot-api/chains/paseo";
import { chainSpec as paseoAssetHubChainSpec } from "polkadot-api/chains/paseo_asset_hub";

export interface ChainSpec {
  name: string;
  id: string;
  chainType: string;
  bootNodes: string[];
  telemetryEndpoints: string[];
  protocolId: string;
  properties: {
    tokenDecimals: number;
    tokenSymbol: string;
  };
  relay_chain: string;
  para_id: number;
  codeSubstitutes: Record<string, string>;
  genesis: {
    stateRootHash: string;
  };
}
export interface ChainConfig {
  key: string;
  name: string;
  descriptors: typeof dot;
  endpoints: string[];
  explorerUrl?: string;
  icon?: React.ReactNode;
  chainSpec: ChainSpec;
  relayChainSpec?: ChainSpec;
}

export type AvailableApis = TypedApi<typeof dot>;

// TODO: add all chains your dapp supports here
export const chainConfig: ChainConfig[] = [
  {
    key: "dot",
    name: "Polkadot",
    descriptors: dot,
    endpoints: ["wss://rpc.polkadot.io"],
    icon: logos.polkadot,
    chainSpec: JSON.parse(polkadotChainSpec),
  },
  {
    key: "polkadot_asset_hub",
    name: "Polkadot Asset Hub",
    descriptors: polkadot_asset_hub,
    endpoints: [
      "wss://polkadot-asset-hub-rpc.polkadot.io",
      "wss://statemint.api.onfinality.io/public-ws",
    ],
    icon: logos.assetHub,
    chainSpec: JSON.parse(polkadotAssetHubChainSpec),
    relayChainSpec: JSON.parse(polkadotChainSpec),
  },
  {
    key: "paseo",
    name: "Paseo",
    descriptors: paseo,
    endpoints: ["wss://rpc.ibp.network/paseo"],
    icon: logos.paseo,
    chainSpec: JSON.parse(paseoChainSpec),
  },
  {
    key: "paseo_asset_hub",
    name: "Paseo Asset Hub",
    descriptors: paseo_asset_hub,
    endpoints: ["wss://asset-hub-paseo-rpc.dwellir.com"],
    icon: logos.paseoAssethub,
    chainSpec: JSON.parse(paseoAssetHubChainSpec),
    relayChainSpec: JSON.parse(paseoChainSpec),
  },
];

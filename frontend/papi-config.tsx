"use client";

import {
  polkadot,
  polkadot_asset_hub,
  paseo,
  paseo_asset_hub,
} from "@polkadot-api/descriptors";
import type { TypedApi } from "polkadot-api";
import { logos } from "@/icons/logos";
import { Computer } from "lucide-react";
import {
  chainSpec,
  chainSpec as polkadotChainSpec,
} from "polkadot-api/chains/polkadot";
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
  descriptors:
    | typeof polkadot
    | typeof polkadot_asset_hub
    | typeof paseo
    | typeof paseo_asset_hub;
  endpoints: string[];
  explorerUrl?: string;
  icon?: React.ReactNode;
  chainSpec: ChainSpec;
  relayChainSpec?: ChainSpec;
}

export type AvailableApis =
  | TypedApi<typeof polkadot>
  | TypedApi<typeof polkadot_asset_hub>
  | TypedApi<typeof paseo>
  | TypedApi<typeof paseo_asset_hub>;

// TODO: add all chains your dapp supports here
export const chainConfig: ChainConfig[] = [
  {
    key: "local",
    name: "Local",
    descriptors: polkadot,
    endpoints: ["ws://localhost:9944"],
    icon: <Computer />,
    chainSpec: JSON.parse(polkadotChainSpec),
  },
  {
    key: "polkadot",
    name: "Polkadot",
    descriptors: polkadot,
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

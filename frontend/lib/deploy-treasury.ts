import { contracts, passethub } from "@polkadot-api/descriptors";
import {
  createReviveSdk,
  getDeploymentAddressWithNonce,
} from "@polkadot-api/sdk-ink";
import { createClient, FixedSizeBinary, HexString } from "polkadot-api";
import { withPolkadotSdkCompat } from "polkadot-api/polkadot-sdk-compat";
import { getWsProvider } from "polkadot-api/ws-provider/web";
import { Binary } from "polkadot-api";
import { InjectedPolkadotAccount } from "polkadot-api/pjs-signer";

// POP Network
// const CONTRACT_NETWORK = "wss://rpc1.paseo.popnetwork.xyz";

// Passet Hub
const CONTRACT_NETWORK = "wss://testnet-passet-hub.polkadot.io";

export async function deployTreasury(
  fromAccount: InjectedPolkadotAccount | null
) {
  if (!fromAccount) {
    throw new Error("No account selected");
  }

  console.log("🛠️ Deploying new treasury from account", fromAccount);

  try {
    if (!fromAccount) {
      throw new Error("No account selected");
    }

    // Fetch WASM file from public directory
    const response = await fetch("/treasury.polkavm");
    if (!response.ok) {
      throw new Error("Failed to load WASM file");
    }

    const wasmBuffer = await response.arrayBuffer();
    const wasmBytes = Binary.fromBytes(new Uint8Array(wasmBuffer));

    // Initialize client
    const client = createClient(
      withPolkadotSdkCompat(getWsProvider(CONTRACT_NETWORK))
    );
    const typedApi = client.getTypedApi(passethub);

    // Initialize contract SDK and deployer
    const treasurySdk = createReviveSdk(typedApi, contracts.treasury);
    const treasuryDeployer = treasurySdk.getDeployer(wasmBytes);

    const contractInitializationOptions = {
      origin: fromAccount.address,
      data: {
        owner: FixedSizeBinary.fromHex(
          "0x00000013100000000000000000000000000000013"
        ),
      },
    };

    const dryRunResult = await treasuryDeployer.dryRun(
      "new",
      contractInitializationOptions
    );

    console.log("dryRunResult", dryRunResult);

    if (!dryRunResult.success) {
      console.error("dryRunResult", dryRunResult);
      throw new Error("Dry run failed");
    }

    // Estimate address using salt:
    const estimatedAddress = await treasuryDeployer.estimateAddress(
      "new",
      contractInitializationOptions
    );

    console.log("estimatedAddress", estimatedAddress);

    if (!estimatedAddress) {
      throw new Error("Failed to estimate contract address");
    }

    // Convert subscription to Promise to properly return values
    const deploymentResult = await new Promise<{
      ss58Address: string;
      contractAddress: HexString;
    }>((resolve, reject) => {
      const subscription = dryRunResult.value
        .deploy()
        .signSubmitAndWatch(fromAccount.polkadotSigner)
        .subscribe({
          next: (txEvent) => {
            console.log("txEvent:", txEvent);
            if (
              txEvent.type === "finalized" ||
              (txEvent.type === "txBestBlocksState" && txEvent.found)
            ) {
              // here we are sure that the transaction is in a block (whether finalized or bestBlock)
              // with `ok` we know the extrinsic failed

              if (txEvent.ok) {
                const newAccountEvent = txEvent.events.find(
                  (event) =>
                    event.type === "System" && event.value.type === "NewAccount"
                );

                if (!newAccountEvent) {
                  subscription.unsubscribe();
                  reject(new Error("New account event not found"));
                  return;
                }

                subscription.unsubscribe();
                resolve({
                  ss58Address: newAccountEvent.value.value.account,
                  contractAddress: estimatedAddress,
                });
              } else {
                console.log("transaction failed");
                const err = txEvent.dispatchError;
                subscription.unsubscribe();
                reject(new Error("Transaction failed", { cause: err }));
              }
            }
          },
          error: (error) => {
            reject(error);
          },
        });
    });

    console.log("deploymentResult", deploymentResult);

    return deploymentResult;
  } catch (err) {
    console.error("Deployment error:", err);
    throw err;
  }
}

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

const CONTRACT_NETWORK = "wss://testnet-passet-hub.polkadot.io";

export interface DeployTreasuryResult {
  ss58Address: string;
  contractAddress: HexString;
}

export interface TreasuryContractService {
  deploy: (
    fromAccount: InjectedPolkadotAccount
  ) => Promise<DeployTreasuryResult>;
  getPendingPayouts: (
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount
  ) => Promise<any>;
  getPendingPayoutIds: (
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount
  ) => Promise<any>;
  addPayout: (
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount,
    to: string,
    amount: bigint
  ) => Promise<any>;
  addPayoutBatch: (
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount,
    payouts: [FixedSizeBinary<20>, bigint][]
  ) => Promise<any>;
  getBalance: (contractAddress: HexString) => Promise<bigint>;
}

class TreasuryContractServiceImpl implements TreasuryContractService {
  private async getClient() {
    return createClient(withPolkadotSdkCompat(getWsProvider(CONTRACT_NETWORK)));
  }

  private async getTreasuryContract(contractAddress: HexString) {
    const client = await this.getClient();
    const typedApi = client.getTypedApi(passethub);
    const treasurySdk = createReviveSdk(typedApi, contracts.treasury);
    return treasurySdk.getContract(contractAddress);
  }

  async deploy(
    fromAccount: InjectedPolkadotAccount
  ): Promise<DeployTreasuryResult> {
    if (!fromAccount) {
      throw new Error("No account selected");
    }

    console.log("🛠️ Deploying new treasury from account", fromAccount);

    try {
      // Fetch WASM file from public directory
      const response = await fetch("/treasury.polkavm");
      if (!response.ok) {
        throw new Error("Failed to load WASM file");
      }

      const wasmBuffer = await response.arrayBuffer();
      const wasmBytes = Binary.fromBytes(new Uint8Array(wasmBuffer));

      // Initialize client
      const client = await this.getClient();
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
      const deploymentResult = await new Promise<DeployTreasuryResult>(
        (resolve, reject) => {
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
                  if (txEvent.ok) {
                    const newAccountEvent = txEvent.events.find(
                      (event) =>
                        event.type === "System" &&
                        event.value.type === "NewAccount"
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
        }
      );

      console.log("deploymentResult", deploymentResult);
      return deploymentResult;
    } catch (err) {
      console.error("Deployment error:", err);
      throw err;
    }
  }

  async getPendingPayouts(
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount
  ): Promise<any> {
    if (!fromAccount) {
      throw new Error("No account selected");
    }

    if (!contractAddress) {
      throw new Error("No contract address provided");
    }

    try {
      const treasuryContract = await this.getTreasuryContract(contractAddress);

      console.log("Getting pending payouts for account", fromAccount.address);
      const result = await treasuryContract.query("get_pending_payouts", {
        origin: fromAccount.address,
      });

      if (result.success) {
        console.log("get_pending_payouts", result.value.response);
        return result.value.response;
      } else {
        throw new Error("Failed to get pending payouts");
      }
    } catch (err) {
      console.error("Get pending payouts error:", err);
      throw err;
    }
  }

  async getPendingPayoutIds(
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount
  ): Promise<any> {
    if (!fromAccount) {
      throw new Error("No account selected");
    }

    if (!contractAddress) {
      throw new Error("No contract address provided");
    }

    try {
      const treasuryContract = await this.getTreasuryContract(contractAddress);

      console.log(
        "Getting pending payout IDs for account",
        fromAccount.address
      );
      const result = await treasuryContract.query("get_pending_payout_ids", {
        origin: fromAccount.address,
      });

      if (result.success) {
        console.log("get_pending_payout_ids", result.value.response);
        return result.value.response;
      } else {
        throw new Error("Failed to get pending payout IDs");
      }
    } catch (err) {
      console.error("Get pending payout IDs error:", err);
      throw err;
    }
  }

  async addPayout(
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount,
    to: string,
    amount: bigint
  ): Promise<any> {
    if (!fromAccount) {
      throw new Error("No account selected");
    }

    if (!contractAddress) {
      throw new Error("No contract address provided");
    }

    try {
      const treasuryContract = await this.getTreasuryContract(contractAddress);

      const dryRunResult = await treasuryContract.query("add_payout", {
        origin: fromAccount.address,
        data: {
          to: FixedSizeBinary.fromHex(to),
          amount: amount,
        },
      });

      if (dryRunResult.success) {
        console.log("add_payout", dryRunResult.value.response);
        console.log("events", dryRunResult.value.events);

        const addPayoutTxResult = await dryRunResult.value
          .send()
          .signAndSubmit(fromAccount.polkadotSigner);

        if (addPayoutTxResult.ok) {
          console.log("block", addPayoutTxResult.block);
          console.log(
            "events",
            treasuryContract.filterEvents(addPayoutTxResult.events)
          );
          return addPayoutTxResult;
        } else {
          console.log("error", addPayoutTxResult.dispatchError);
          throw new Error("Transaction failed");
        }
      } else {
        console.log("error", dryRunResult.value);
        throw new Error("Dry run failed");
      }
    } catch (err) {
      console.error("Add payout error:", err);
      throw err;
    }
  }

  async addPayoutBatch(
    contractAddress: HexString,
    fromAccount: InjectedPolkadotAccount,
    payouts: [FixedSizeBinary<20>, bigint][]
  ): Promise<any> {
    if (!fromAccount) {
      throw new Error("No account selected");
    }

    if (!contractAddress) {
      throw new Error("No contract address provided");
    }

    try {
      const treasuryContract = await this.getTreasuryContract(contractAddress);

      const dryRunResult = await treasuryContract.query("add_payout_batch", {
        origin: fromAccount.address,
        data: {
          payouts,
        },
      });

      if (dryRunResult.success) {
        console.log("add_payout_batch", dryRunResult.value.response);
        console.log("events", dryRunResult.value.events);

        const addPayoutBatchTxResult = await dryRunResult.value
          .send()
          .signSubmitAndWatch(fromAccount.polkadotSigner)
          .subscribe((txEvent) => {
            console.log("txEvent:", txEvent);
            if (
              txEvent.type === "finalized" ||
              (txEvent.type === "txBestBlocksState" && txEvent.found)
            ) {
              if (txEvent.ok) {
                console.log("block", txEvent.block);
                console.log(
                  "events",
                  treasuryContract.filterEvents(txEvent.events)
                );
              } else {
                console.log("error", txEvent.dispatchError);
              }
            }
          });

        return addPayoutBatchTxResult;
      } else {
        console.log("error", dryRunResult.value);
        throw new Error("Dry run failed");
      }
    } catch (err) {
      console.error("Add payout batch error:", err);
      throw err;
    }
  }

  async getBalance(contractAddress: HexString): Promise<bigint> {
    if (!contractAddress) {
      throw new Error("No contract address provided");
    }

    try {
      const treasuryContract = await this.getTreasuryContract(contractAddress);

      console.log("Getting contract balance for", contractAddress);
      const result = await treasuryContract.query("get_balance", {
        origin: contractAddress, // Use contract address as origin for the query
      });

      if (result.success) {
        console.log("get_balance result:", result.value.response);
        return result.value.response as bigint;
      } else {
        console.warn("Failed to get contract balance:", result.value);
        return BigInt(0);
      }
    } catch (err) {
      console.error(
        "Get balance error for contract",
        contractAddress,
        ":",
        err
      );

      // Handle various contract errors gracefully
      if (err instanceof Error) {
        if (
          err.message.includes("checksum") ||
          err.message.includes("Invalid")
        ) {
          console.warn("Invalid contract address format:", contractAddress);
          return BigInt(0);
        }
        if (
          err.message.includes("Contract not found") ||
          err.message.includes("does not exist")
        ) {
          console.warn("Contract does not exist:", contractAddress);
          return BigInt(0);
        }
      }

      // For other errors, still return 0 to prevent UI crashes
      console.warn("Returning 0 balance due to error");
      return BigInt(0);
    }
  }
}

// Export singleton instance
export const treasuryContractService = new TreasuryContractServiceImpl();

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod treasury {

    use ink::prelude::vec::Vec;
    use ink::H160;
    use ink::{prelude::collections::BTreeMap, prelude::vec, U256};
    use parity_scale_codec::{Decode, Encode};

    #[cfg(feature = "debug")]
    use ink::prelude::{borrow::ToOwned, format, string::String};

    #[ink::event]
    #[cfg(feature = "debug")]
    pub struct DebugEvent {
        message: String,
    }

    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PendingPayout {
        to: H160,
        amount: U256,
        block_number: u32,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Treasury {
        /// The owner of the contract
        owner: H160,
        /// The treasurers of the contract
        treasurers: Vec<H160>,
        /// Pending payouts
        pending_payouts: Vec<PendingPayout>,
        /// Payout frequency in blocks
        payout_frequency: u32,
        /// Last block when payouts were processed
        last_payout_processed: u32,
        /// The last payout block of the contract
        last_payout_block: u32,
    }

    /// Custom errors for the treasury contract
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    #[repr(u8)]
    pub enum Error {
        /// Caller is not the owner
        NotOwner = 0,
        /// Caller is not a treasurer
        NotTreasurer = 1,
        /// Insufficient treasury balance
        InsufficientTreasuryBalance = 2,
        /// Invalid payout frequency
        InvalidFrequency = 3,
        /// Payouts already processed recently
        TooEarlyToProcess = 4,
        /// Treasurer already exists
        TreasurerExists = 5,
    }

    /// Type alias for the contract's result type
    pub type Result<T> = core::result::Result<T, Error>;

    /// Events emitted by the contract
    #[ink(event)]
    pub struct TreasurerAdded {
        #[ink(topic)]
        treasurer: H160,
    }

    #[ink(event)]
    pub struct TreasurerRemoved {
        #[ink(topic)]
        treasurer: H160,
    }

    #[ink(event)]
    pub struct PayoutAdded {
        #[ink(topic)]
        to: H160,
        amount: U256,
    }

    #[ink(event)]
    pub struct PayoutsProcessed {
        total_amount: U256,
        payouts_count: u32,
    }

    #[ink(event)]
    pub struct PayoutFrequencyChanged {
        old_frequency: u32,
        new_frequency: u32,
    }

    impl Treasury {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(payout_frequency: u32) -> Self {
            let caller = Self::env().caller();

            Self {
                owner: caller,
                treasurers: vec![caller],
                pending_payouts: vec![],
                payout_frequency: payout_frequency,
                last_payout_block: 0,
                last_payout_processed: 0,
            }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// Get the current address
        #[ink(message)]
        pub fn get_address(&self) -> AccountId {
            self.env().account_id()
        }

        /// Get the current balance
        #[ink(message)]
        pub fn get_balance(&self) -> U256 {
            self.env().balance()
        }

        /// Get the list of treasurers
        #[ink(message)]
        pub fn get_treasurers(&self) -> Vec<H160> {
            self.treasurers.clone()
        }

        /// Get the payout frequency
        #[ink(message)]
        pub fn get_payout_frequency(&self) -> u32 {
            self.payout_frequency
        }

        /// Get the pending payouts
        #[ink(message)]
        pub fn get_pending_payouts(&self) -> Vec<PendingPayout> {
            self.pending_payouts.clone()
        }

        /// Add a new treasurer
        #[ink(message)]
        pub fn add_treasurer(&mut self, treasurer: H160) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }

            // #[cfg(feature = "debug")]
            // self.env().emit_event(DebugEvent {
            //     message: format!(
            //         "received {:?} to address {:?}",
            //         self.env().balance(),
            //         self.env().account_id()
            //     )
            //     .to_owned(),
            // });

            if self.treasurers.contains(&treasurer) {
                return Err(Error::TreasurerExists);
            }

            self.treasurers.push(treasurer);
            self.env().emit_event(TreasurerAdded { treasurer });

            Ok(())
        }

        /// Remove a treasurer
        #[ink(message)]
        pub fn remove_treasurer(&mut self, treasurer: H160) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }

            if let Some(pos) = self.treasurers.iter().position(|&x| x == treasurer) {
                self.treasurers.remove(pos);
                self.env().emit_event(TreasurerRemoved { treasurer });
            }

            Ok(())
        }

        /// Add a new payout
        #[ink(message)]
        pub fn add_payout(&mut self, to: H160, amount: U256) -> Result<()> {
            #[cfg(feature = "std")]
            if !self.treasurers.contains(&self.env().caller()) {
                #[cfg(feature = "std")]
                return Err(Error::NotTreasurer);
            }

            let balance = self.env().balance();
            if balance < amount {
                return Err(Error::InsufficientTreasuryBalance);
            }

            self.pending_payouts.push(PendingPayout {
                to,
                amount,
                block_number: self.env().block_number(),
            });

            self.env().emit_event(PayoutAdded { to, amount });

            Ok(())
        }

        /// Process pending payouts
        #[ink(message)]
        pub fn process_pending_payouts(&mut self) -> Result<()> {
            let current_block = self.env().block_number();

            let next_payout_block = self
                .last_payout_processed
                .saturating_add(self.payout_frequency);

            if current_block < next_payout_block {
                return Err(Error::TooEarlyToProcess);
            }

            let mut aggregated_payouts: BTreeMap<H160, U256> = BTreeMap::new();

            for payout in &self.pending_payouts {
                let current = aggregated_payouts
                    .get(&payout.to)
                    .copied()
                    .unwrap_or(U256::from(0));
                aggregated_payouts.insert(payout.to, current.saturating_add(payout.amount));
            }

            let mut total_amount: U256 = U256::from(0);
            let mut payouts_count: u32 = 0;

            for (to, amount) in aggregated_payouts {
                #[cfg(feature = "std")]
                if self.env().transfer(to, amount).is_err() {
                } else {
                    total_amount = total_amount.saturating_add(amount);
                    payouts_count = payouts_count.saturating_add(1);
                }
            }

            self.pending_payouts.clear();
            self.last_payout_block = current_block;

            self.env().emit_event(PayoutsProcessed {
                total_amount,
                payouts_count,
            });

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let my_contract = Treasury::default();
            assert_eq!(my_contract.payout_frequency, 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut my_contract = Treasury::new(0);
            assert_eq!(my_contract.payout_frequency, 0);

            let someone = H160::random();

            my_contract.add_payout(someone, U256::from(100));
            assert_eq!(my_contract.pending_payouts.len(), 1);
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TreasuryRef::default();

            // When
            let contract = client
                .instantiate("my_contract", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Treasury>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TreasuryRef::new(false);
            let contract = client
                .instantiate("my_contract", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Treasury>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::{debug_println, DefaultEnvironment};
use ink::prelude::{collections::BTreeMap, collections::BTreeSet, string::String, vec::Vec};
use parity_scale_codec::{Decode, Encode};
use pop_api::{
    primitives::TokenId,
    v0::fungibles::{
        self as api,
        // events::{Approval, Created, Transfer},
        // traits::{Psp22, Psp22Burnable, Psp22Metadata, Psp22Mintable},
        // Psp22Error,
    },
};

#[cfg(feature = "std")]
fn format_account_id(account_id: &ink::primitives::AccountId) -> String {
    let bytes: &[u8] = account_id.as_ref();
    format!(
        "{:?}",
        ink::primitives::AccountId::decode(&mut &bytes[..])
            .unwrap_or(ink::primitives::AccountId([0; 32]))
    )
}

#[ink::contract]
mod treasury {
    use super::*;

    /// Represents a payout
    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Payout {
        /// Unique identifier for the payout
        id: u32,
        /// Recipient of the payout
        to: AccountId,
        /// Amount to be paid out
        amount: Balance,
        /// Block number when payout was added
        block_number: BlockNumber,
        /// Treasurers who approved this payout
        approvals: Vec<AccountId>,
        /// Type of payout
        payout_type: PayoutType,
        /// Status of the payout
        status: PayoutStatus,
        /// For recurring/vested: interval between payouts in blocks (0 for one-time)
        interval_blocks: BlockNumber,
        /// For recurring/vested: total number of payouts (0 for one-time)
        total_payouts: u32,
        /// For recurring/vested: number of payouts completed
        completed_payouts: u32,
        /// For vested: cliff period in blocks (0 for no cliff)
        cliff_blocks: BlockNumber,
        /// Treasurers who approved this cancellation
        cancellation_approvals: Vec<AccountId>,
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PayoutType {
        OneTime = 0,
        Recurring = 1,
        Vested = 2,
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PayoutStatus {
        Pending = 0,
        Active = 1,
        Completed = 2,
        Cancelled = 3,
    }

    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Balances {
        pub native: Balance,
        pub assets: Vec<(TokenId, Balance)>,
    }

    /// Represents a threshold for treasurer approvals
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Threshold {
        /// Minimum amount for this threshold
        min_amount: Balance,
        /// Maximum amount for this threshold
        max_amount: Balance,
        /// Required number of treasurer approvals
        required_approvals: u32,
    }

    /// Defines the storage of your treasury contract.
    #[ink(storage)]
    pub struct Treasury {
        /// Owner of the treasury
        owner: AccountId,
        /// List of treasurers who can add payouts
        treasurers: BTreeSet<AccountId>,
        /// Pending payouts
        pending_payouts: Vec<Payout>,
        /// Past payouts
        past_payouts: Vec<Payout>,
        /// List of registered asset ids
        registered_assets: BTreeSet<TokenId>,
        /// Thresholds for treasurer approvals
        thresholds: Vec<Threshold>,
        /// Reentrancy guard
        processing: bool,
    }

    /// Maximum number of past payouts to keep in storage
    const MAX_PAST_PAYOUTS: usize = 1000;

    /// Custom errors for the treasury contract
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    #[repr(u8)]
    pub enum Error {
        /// Caller is not the owner
        NotOwner = 0,
        /// Caller is not a treasurer
        NotTreasurer = 1,
        /// Insufficient balance
        InsufficientBalance = 2,
        /// Invalid payout frequency
        InvalidFrequency = 3,
        /// Payouts already processed recently
        TooEarlyToProcess = 4,
        /// Treasurer already exists
        TreasurerExists = 5,
        /// Payout not found
        PayoutNotFound = 6,
        /// Reentrancy detected
        Reentrancy = 7,
    }

    /// Type alias for the contract's result type
    pub type Result<T> = core::result::Result<T, Error>;

    /// Events emitted by the contract
    #[ink(event)]
    pub struct TreasurerAdded {
        #[ink(topic)]
        treasurer: AccountId,
    }

    #[ink(event)]
    pub struct TreasurerRemoved {
        #[ink(topic)]
        treasurer: AccountId,
    }

    #[ink(event)]
    pub struct PayoutAdded {
        #[ink(topic)]
        to: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct PayoutCreated {
        payout_id: u32,
        recipient: AccountId,
        amount: Balance,
        start_block: BlockNumber,
    }

    #[ink(event)]
    pub struct PayoutsProcessed {
        total_amount: Balance,
        payouts_count: u32,
    }

    #[ink(event)]
    pub struct PayoutCancelled {
        #[ink(topic)]
        payout_id: u32,
    }

    #[ink(event)]
    pub struct PayoutFrequencyChanged {
        payout_id: u32,
        old_frequency: BlockNumber,
        new_frequency: BlockNumber,
    }

    impl Treasury {
        /// Constructor that creates a new treasury with the caller as owner
        #[ink(constructor)]
        pub fn new() -> Result<Self> {
            let mut thresholds = Vec::new();
            thresholds.push(Threshold {
                min_amount: 0,
                max_amount: 500_000_000_000,
                required_approvals: 1,
            });
            thresholds.push(Threshold {
                min_amount: 500_000_000_000,
                max_amount: 2_500_000_000_000,
                required_approvals: 2,
            });
            thresholds.push(Threshold {
                min_amount: 2_500_000_000_000,
                max_amount: Balance::MAX,
                required_approvals: 3,
            });

            Ok(Self {
                owner: Self::env().caller(),
                treasurers: BTreeSet::new(),
                pending_payouts: Vec::new(),
                past_payouts: Vec::new(),
                registered_assets: BTreeSet::new(),
                thresholds,
                processing: false,
            })
        }

        /// Add a new treasurer
        #[ink(message)]
        pub fn add_treasurer(&mut self, treasurer: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }

            if !self.treasurers.insert(treasurer) {
                #[cfg(feature = "std")]
                debug_println!(
                    "Treasurer already exists: {}",
                    format_account_id(&treasurer)
                );
                return Err(Error::TreasurerExists);
            }

            self.env().emit_event(TreasurerAdded { treasurer });

            Ok(())
        }

        /// Remove a treasurer
        #[ink(message)]
        pub fn remove_treasurer(&mut self, treasurer: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }

            if self.treasurers.remove(&treasurer) {
                self.env().emit_event(TreasurerRemoved { treasurer });
            }

            Ok(())
        }

        /// Add a new one-time payout
        #[ink(message)]
        pub fn add_payout(&mut self, to: AccountId, amount: Balance) -> Result<u32> {
            self.add_payout_internal(to, amount, PayoutType::OneTime, 0, 0, 0)
        }

        /// Add a new recurring payout
        #[ink(message)]
        pub fn add_recurring_payout(
            &mut self,
            to: AccountId,
            amount: Balance,
            interval_blocks: BlockNumber,
            total_payouts: u32,
        ) -> Result<u32> {
            self.add_payout_internal(
                to,
                amount,
                PayoutType::Recurring,
                interval_blocks,
                total_payouts,
                0,
            )
        }

        /// Add a new vested payout
        #[ink(message)]
        pub fn add_vested_payout(
            &mut self,
            to: AccountId,
            amount: Balance,
            interval_blocks: BlockNumber,
            total_payouts: u32,
            cliff_blocks: BlockNumber,
        ) -> Result<u32> {
            self.add_payout_internal(
                to,
                amount,
                PayoutType::Vested,
                interval_blocks,
                total_payouts,
                cliff_blocks,
            )
        }

        /// Internal function to add any type of payout
        fn add_payout_internal(
            &mut self,
            to: AccountId,
            amount: Balance,
            payout_type: PayoutType,
            interval_blocks: BlockNumber,
            total_payouts: u32,
            cliff_blocks: BlockNumber,
        ) -> Result<u32> {
            #[cfg(feature = "std")]
            debug_println!("Adding payout: {} to {}", amount, format_account_id(&to));

            if !self.treasurers.contains(&self.env().caller()) {
                return Err(Error::NotTreasurer);
            }

            let balance = self.env().balance();
            if balance < amount {
                debug_println!("Insufficient balance: {} < {}", balance, amount);
                return Err(Error::InsufficientBalance);
            }

            // Remove oldest payout if we exceed MAX_PAST_PAYOUTS
            if self.past_payouts.len() >= MAX_PAST_PAYOUTS {
                self.past_payouts.remove(0);
                debug_println!("Removed oldest payout to maintain size limit");
            }

            // Generate new payout ID
            let payout_id = u32::try_from(self.pending_payouts.len())
                .map(|len| len.saturating_add(1))
                .unwrap_or(1);

            // Add new payout with initial approval
            let mut approvals = Vec::new();
            approvals.push(self.env().caller());
            self.pending_payouts.push(Payout {
                id: payout_id,
                to,
                amount,
                block_number: self.env().block_number(),
                approvals,
                payout_type,
                status: PayoutStatus::Pending,
                interval_blocks,
                total_payouts,
                completed_payouts: 0,
                cliff_blocks,
                cancellation_approvals: Vec::new(), // Initialize cancellation approvals
            });

            self.env().emit_event(PayoutAdded { to, amount });

            Ok(payout_id)
        }

        /// Approve a pending payout
        #[ink(message)]
        pub fn approve(&mut self, payout_id: u32) -> Result<()> {
            if !self.treasurers.contains(&self.env().caller()) {
                return Err(Error::NotTreasurer);
            }

            let caller = self.env().caller();
            if let Some(payout) = self.pending_payouts.iter_mut().find(|p| p.id == payout_id) {
                if !payout.approvals.contains(&caller) {
                    debug_println!("Approved payout: {}", payout_id);
                    payout.approvals.push(caller);
                }
                return Ok(());
            }

            Err(Error::PayoutNotFound)
        }

        /// Process pending payouts
        #[ink(message)]
        pub fn process_pending_payouts(&mut self) -> Result<()> {
            // Reentrancy guard
            if self.processing {
                return Err(Error::Reentrancy);
            }
            self.processing = true;

            debug_println!("Processing {} pending payouts", self.pending_payouts.len());
            let current_block = self.env().block_number();

            let mut processed_payouts = Vec::new();
            let mut remaining_payouts = Vec::new();
            let payouts: Vec<_> = self.pending_payouts.drain(..).collect();

            for payout in payouts {
                // Get required approvals for this amount
                let required_approvals = self
                    .thresholds
                    .iter()
                    .find(|t| payout.amount >= t.min_amount && payout.amount <= t.max_amount)
                    .map(|t| t.required_approvals)
                    .unwrap_or(1)
                    .min(u32::try_from(self.treasurers.len()).unwrap_or(u32::MAX));

                debug_println!(
                    "Processing payout: {} with {} approvals, required {}",
                    payout.id,
                    payout.approvals.len(),
                    required_approvals
                );

                // Skip if not enough approvals
                if payout.approvals.len() < required_approvals as usize {
                    remaining_payouts.push(payout);
                    continue;
                }

                // Check if payout is ready to be processed
                let is_ready = match payout.payout_type {
                    PayoutType::OneTime => true,
                    PayoutType::Recurring => {
                        let next_payout_block = payout
                            .block_number
                            .checked_add(
                                payout
                                    .completed_payouts
                                    .checked_mul(payout.interval_blocks)
                                    .unwrap_or(0),
                            )
                            .unwrap_or(0);
                        current_block >= next_payout_block
                    }
                    PayoutType::Vested => {
                        let cliff_block = payout
                            .block_number
                            .checked_add(payout.cliff_blocks)
                            .unwrap_or(0);
                        let next_payout_block = payout
                            .block_number
                            .checked_add(
                                payout
                                    .completed_payouts
                                    .checked_mul(payout.interval_blocks)
                                    .unwrap_or(0),
                            )
                            .unwrap_or(0);
                        current_block >= cliff_block && current_block >= next_payout_block
                    }
                };

                if !is_ready {
                    remaining_payouts.push(payout);
                    continue;
                }

                // Update state before transfer
                let mut processed_payout = payout.clone();
                processed_payout.status = PayoutStatus::Completed;
                processed_payout.completed_payouts = processed_payout
                    .completed_payouts
                    .checked_add(1)
                    .unwrap_or(0);

                // Process the payout
                debug_println!("Attempting transfer: {} to {:?}", payout.amount, payout.to);
                let transfer_result = self.env().transfer(payout.to, payout.amount);
                debug_println!("Transfer result: {:?}", transfer_result);

                if transfer_result.is_ok() {
                    debug_println!("Transfer successful for payout: {}", payout.id);

                    // If it's a recurring/vested payout and not all payouts are completed,
                    // add it back to pending with updated status
                    if (processed_payout.payout_type == PayoutType::Recurring
                        || processed_payout.payout_type == PayoutType::Vested)
                        && processed_payout.completed_payouts < processed_payout.total_payouts
                    {
                        processed_payout.status = PayoutStatus::Active;
                        remaining_payouts.push(processed_payout);
                    } else {
                        debug_println!(
                            "Adding payout {} to processed_payouts",
                            processed_payout.id
                        );
                        processed_payouts.push(processed_payout);
                    }
                } else {
                    debug_println!("Transfer failed for payout: {}", payout.id);
                    remaining_payouts.push(payout);
                }
            }

            // Update pending and past payouts
            debug_println!("Moving {} payouts to past", processed_payouts.len());
            self.pending_payouts = remaining_payouts;
            for payout in processed_payouts {
                debug_println!("Adding payout {} to past_payouts", payout.id);
                if self.past_payouts.len() >= MAX_PAST_PAYOUTS {
                    self.past_payouts.remove(0);
                }
                self.past_payouts.push(payout);
            }
            debug_println!(
                "Past payouts length after update: {}",
                self.past_payouts.len()
            );

            // Reset reentrancy guard
            self.processing = false;

            Ok(())
        }

        /// Register a new asset id the contract tracks and handles
        #[ink(message)]
        pub fn register_asset_id(&mut self, asset_id: TokenId) -> Result<()> {
            debug_println!("Registering asset id: {}", asset_id);
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            if !self.registered_assets.insert(asset_id) {
                return Ok(()); // already registered, no-op
            }
            debug_println!("Registered asset id: {}", asset_id);
            Ok(())
        }

        /// Get all balances: native and registered assets
        #[ink(message)]
        pub fn get_balances(&self) -> Balances {
            let native = self.env().balance();
            let mut assets = Vec::new();
            let contract = self.env().account_id();
            for asset_id in &self.registered_assets {
                let bal = api::balance_of(*asset_id, contract).unwrap_or(0);
                assets.push((*asset_id, bal));
            }
            Balances { native, assets }
        }

        /// Get the current balance
        #[ink(message)]
        pub fn get_balance(&self, asset_id: TokenId) -> Balance {
            if asset_id == 0 {
                return self.env().balance();
            }

            if !self.registered_assets.contains(&asset_id) {
                return 0;
            }

            let contract = self.env().account_id();
            api::balance_of(asset_id, contract).unwrap_or(0)
        }

        /// Get the list of treasurers
        #[ink(message)]
        pub fn get_treasurers(&self) -> Vec<AccountId> {
            self.treasurers.iter().cloned().collect()
        }

        /// Get the pending payouts
        #[ink(message)]
        pub fn get_pending_payouts(&self) -> Vec<Payout> {
            self.pending_payouts.clone()
        }

        /// Cancel a payout (requires treasurer threshold approval)
        #[ink(message)]
        pub fn cancel_payout(&mut self, payout_id: u32) -> Result<()> {
            let caller = self.env().caller();
            if !self.treasurers.contains(&caller) {
                return Err(Error::NotTreasurer);
            }

            if let Some(index) = self.pending_payouts.iter().position(|p| p.id == payout_id) {
                // Get amount and required approvals *before* mutable borrow
                let amount = self.pending_payouts[index].amount;
                let required_approvals = self.get_required_approvals(amount);

                // Get mutable reference to the payout
                let payout = &mut self.pending_payouts[index];

                // Add cancellation approval if not already present
                if !payout.cancellation_approvals.contains(&caller) {
                    payout.cancellation_approvals.push(caller);
                }

                // Check if cancellation threshold is met (using pre-calculated value)
                if payout.cancellation_approvals.len() >= required_approvals as usize {
                    // Threshold met, remove the payout
                    self.pending_payouts.remove(index);
                    self.env().emit_event(PayoutCancelled { payout_id });
                    debug_println!("Payout {} cancelled by threshold.", payout_id);
                } else {
                    debug_println!(
                        "Cancellation approval added for payout {}. Required: {}, Current: {}",
                        payout_id,
                        required_approvals,
                        payout.cancellation_approvals.len()
                    );
                }
                Ok(())
            } else {
                Err(Error::PayoutNotFound)
            }
        }

        /// Get a payout schedule
        #[ink(message)]
        pub fn get_payout_schedule(&self, payout_id: u32) -> Option<Payout> {
            // Stub implementation
            None
        }

        /// Get active payouts
        #[ink(message)]
        pub fn get_active_payouts(&self) -> Vec<Payout> {
            // Stub implementation
            Vec::new()
        }

        /// Get completed payouts
        #[ink(message)]
        pub fn get_completed_payouts(&self) -> Vec<Payout> {
            // Stub implementation
            Vec::new()
        }

        /// Get vested amount
        #[ink(message)]
        pub fn get_vested_amount(&self, payout_id: u32) -> Balance {
            // Stub implementation
            0
        }

        /// Get next payout block
        #[ink(message)]
        pub fn get_next_payout_block(&self, payout_id: u32) -> Option<BlockNumber> {
            // Stub implementation
            None
        }

        /// Get the past payouts
        #[ink(message)]
        pub fn get_past_payouts(&self) -> Vec<Payout> {
            debug_println!("Getting past payouts, length: {}", self.past_payouts.len());
            self.past_payouts.clone()
        }

        #[ink(message)]
        pub fn get_payouts_by_block_range(
            &self,
            start_block: BlockNumber,
            end_block: BlockNumber,
        ) -> Vec<Payout> {
            let mut result = Vec::new();

            // Add payouts from past_payouts
            for payout in &self.past_payouts {
                if payout.block_number >= start_block && payout.block_number <= end_block {
                    result.push(payout.clone());
                }
            }

            result
        }

        /// Get required approvals for an amount
        #[ink(message)]
        pub fn get_required_approvals(&self, amount: Balance) -> u32 {
            self.thresholds
                .iter()
                .find(|t| amount >= t.min_amount && amount <= t.max_amount)
                .map(|t| t.required_approvals)
                .unwrap_or(1)
                .min(u32::try_from(self.treasurers.len()).unwrap_or(u32::MAX))
        }
    }

    /// Unit tests for the treasury contract
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        const ONE_NATIVE_TOKEN: Balance = 10_000_000_000; // 1 native token in units
        const INITIAL_BALANCE: Balance = ONE_NATIVE_TOKEN * 5000;

        /// Helper to set up a treasury with some funds
        fn setup() -> Treasury {
            let mut treasury = Treasury::new().unwrap();

            // Set contract balance
            test::set_account_balance::<DefaultEnvironment>(
                test::callee::<DefaultEnvironment>(),
                INITIAL_BALANCE,
            );

            // Add a treasurer
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();
            treasury.add_treasurer(accounts.bob).unwrap();

            treasury
        }

        /// Test constructor
        #[ink::test]
        fn setup_works() {
            let treasury = setup();
            assert_eq!(treasury.get_balance(0), INITIAL_BALANCE);
        }

        /// Test adding a treasurer
        #[ink::test]
        fn add_treasurer_works() {
            let mut treasury = setup();
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();

            treasury.add_treasurer(accounts.charlie).unwrap();
            assert_eq!(
                treasury.get_treasurers(),
                vec![accounts.bob, accounts.charlie]
            );
        }

        /// Test adding a payout
        #[ink::test]
        fn add_payout_works() {
            let mut treasury = setup();
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();
            test::set_caller::<DefaultEnvironment>(accounts.bob);

            // Add payout
            treasury.add_payout(accounts.charlie, 999).unwrap();

            // // Check balances
            assert_eq!(treasury.get_balance(0), INITIAL_BALANCE);
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(treasury.get_pending_payouts()[0].amount, 999);
            assert_eq!(treasury.get_pending_payouts()[0].to, accounts.charlie);
        }

        /// Test processing payouts
        #[ink::test]
        fn process_payouts_works() {
            let mut treasury = setup();
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();

            // Set Bob as caller (treasurer)
            test::set_caller::<DefaultEnvironment>(accounts.bob);

            // Set initial balance for Test Accounts Charlie and Django
            test::set_account_balance::<DefaultEnvironment>(accounts.charlie, 0);
            test::set_account_balance::<DefaultEnvironment>(accounts.django, 0);

            // Set a high block number to avoid cleanup
            test::set_block_number::<DefaultEnvironment>(1_000_000);

            // Add two payouts to same recipient
            treasury.add_payout(accounts.charlie, 100).unwrap();
            treasury.add_payout(accounts.django, 200).unwrap();
            treasury.add_payout(accounts.charlie, 300).unwrap();

            assert_eq!(treasury.get_pending_payouts().len(), 3);
            assert_eq!(treasury.get_pending_payouts()[0].amount, 100);
            assert_eq!(treasury.get_pending_payouts()[1].amount, 200);
            assert_eq!(treasury.get_pending_payouts()[2].amount, 300);

            // Process payouts
            treasury.process_pending_payouts().unwrap();

            // Verify payouts were moved from pending to past
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            let past_payouts = treasury.get_past_payouts();
            debug_println!("Past payouts length in test: {}", past_payouts.len());
            assert_eq!(past_payouts.len(), 3);

            // Verify the amounts in past_payouts
            let total_amount = past_payouts.iter().map(|p| p.amount).sum::<Balance>();
            assert_eq!(total_amount, 600); // 100 + 200 + 300

            assert_eq!(treasury.get_balance(0), INITIAL_BALANCE - 600);
            assert_eq!(
                test::get_account_balance::<DefaultEnvironment>(accounts.charlie),
                Ok(400)
            );
            assert_eq!(
                test::get_account_balance::<DefaultEnvironment>(accounts.django),
                Ok(200)
            );
        }

        /// Test cutoff blocks functionality
        #[ink::test]
        fn cutoff_blocks_works() {
            let mut treasury = setup();
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();

            // Set caller as treasurer (bob)
            test::set_caller::<DefaultEnvironment>(accounts.bob);

            // Add payouts up to MAX_PAST_PAYOUTS
            for i in 0..MAX_PAST_PAYOUTS {
                treasury
                    .add_payout(accounts.charlie, 100 + i as u128)
                    .unwrap();
                treasury.process_pending_payouts().unwrap();
            }

            // Verify we have exactly MAX_PAST_PAYOUTS entries
            assert_eq!(treasury.get_past_payouts().len(), MAX_PAST_PAYOUTS);

            // Add one more payout
            treasury.add_payout(accounts.charlie, 500).unwrap();
            treasury.process_pending_payouts().unwrap();

            // Verify oldest payout was removed and new one added
            let past_payouts = treasury.get_past_payouts();
            assert_eq!(past_payouts.len(), MAX_PAST_PAYOUTS);
            assert_eq!(past_payouts[0].amount, 101); // Second payout (first was removed)
            assert_eq!(past_payouts[MAX_PAST_PAYOUTS - 1].amount, 500); // New payout
        }

        /// Test threshold requirements
        #[ink::test]
        fn threshold_requirements_work() {
            let mut treasury = setup();
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();

            // Test small amount (1 USD) - requires 1 approval
            assert_eq!(treasury.get_required_approvals(1 * ONE_NATIVE_TOKEN), 1);

            // Test medium amount (1000 USD) - requires 2 approvals but there is only 1 treasurer
            assert_eq!(treasury.get_required_approvals(1000 * ONE_NATIVE_TOKEN), 1);

            // Add more treasurers
            treasury.add_treasurer(accounts.charlie).unwrap();

            // Test large amount (3000 USD) - requires 3 approvals but there is only 2 treasurers
            assert_eq!(treasury.get_required_approvals(3000 * ONE_NATIVE_TOKEN), 2);

            treasury.add_treasurer(accounts.django).unwrap();

            // Test with more treasurers available
            assert_eq!(treasury.get_required_approvals(3000 * ONE_NATIVE_TOKEN), 3);
        }

        /// Test multi-treasurer approval process
        #[ink::test]
        fn multi_treasurer_approval_works() {
            let mut treasury = setup();
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();

            // Add more treasurers
            treasury.add_treasurer(accounts.charlie).unwrap();
            treasury.add_treasurer(accounts.django).unwrap();

            // Set initial balance for recipient
            test::set_account_balance::<DefaultEnvironment>(accounts.eve, 0);

            // Add a medium-sized payout (1000 USD) - requires 2 approvals
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            let payout_id = treasury
                .add_payout(accounts.eve, 1000 * ONE_NATIVE_TOKEN)
                .unwrap();

            // Try to process with only 1 approval - should not process
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            treasury.process_pending_payouts().unwrap();
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(
                test::get_account_balance::<DefaultEnvironment>(accounts.eve),
                Ok(0)
            );

            // Add second approval
            test::set_caller::<DefaultEnvironment>(accounts.charlie);
            treasury.approve(payout_id).unwrap();

            // Process with 2 approvals - should fail
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            treasury.process_pending_payouts().unwrap();

            assert_eq!(treasury.get_pending_payouts().len(), 1);

            // Process with 3 approvals - should succeed
            test::set_caller::<DefaultEnvironment>(accounts.django);
            treasury.approve(payout_id).unwrap();

            test::set_caller::<DefaultEnvironment>(accounts.bob);
            treasury.process_pending_payouts().unwrap();

            assert_eq!(
                test::get_account_balance::<DefaultEnvironment>(accounts.eve),
                Ok(1000 * ONE_NATIVE_TOKEN)
            );
        }

        /// Test large payout requiring all treasurers
        #[ink::test]
        fn large_payout_requires_all_treasurers() {
            let mut treasury = setup();
            let accounts: test::DefaultAccounts<DefaultEnvironment> = test::default_accounts();

            // Add more treasurers
            treasury.add_treasurer(accounts.charlie).unwrap();
            treasury.add_treasurer(accounts.django).unwrap();

            // Set initial balance for recipient
            test::set_account_balance::<DefaultEnvironment>(accounts.eve, 0);

            // Add a large payout (3000 USD) - requires 3 approvals
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            let payout_id = treasury
                .add_payout(accounts.eve, 3000 * ONE_NATIVE_TOKEN)
                .unwrap();

            // Add second approval
            test::set_caller::<DefaultEnvironment>(accounts.charlie);
            treasury.approve(payout_id).unwrap();

            // Try to process with 2 approvals - should not process
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            treasury.process_pending_payouts().unwrap();
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(
                test::get_account_balance::<DefaultEnvironment>(accounts.eve),
                Ok(0)
            );

            // Add third approval
            test::set_caller::<DefaultEnvironment>(accounts.django);
            treasury.approve(payout_id).unwrap();

            // Process with 3 approvals - should succeed
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            treasury.process_pending_payouts().unwrap();
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            assert_eq!(
                test::get_account_balance::<DefaultEnvironment>(accounts.eve),
                Ok(3000 * ONE_NATIVE_TOKEN)
            );
        }
    }
}

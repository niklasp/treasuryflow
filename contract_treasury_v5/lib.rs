#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::{DefaultEnvironment, Environment};
use ink::prelude::{collections::BTreeMap, string::String, vec::Vec};
use parity_scale_codec::{Decode, Encode};
use pop_api::{
    primitives::TokenId,
    v0::fungibles::{
        self as api,
        events::{Approval, Created, Transfer},
        traits::{Psp22, Psp22Burnable, Psp22Metadata, Psp22Mintable},
        Psp22Error,
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

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PayoutSchedule {
        pub recipient: AccountId,
        pub amount: Balance,
        pub start_block: BlockNumber,
        pub end_block: BlockNumber,
        pub payout_type: PayoutType,
        pub status: PayoutStatus,
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PayoutType {
        Scheduled = 0,
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

    /// Represents a pending payout
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PendingPayout {
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

    /// Represents a past payout
    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PastPayout {
        /// Recipient of the payout
        to: AccountId,
        /// Amount that was paid out
        amount: Balance,
        /// Asset ID that was paid out
        asset_id: TokenId,
        /// Block number when payout was added
        payout_block: BlockNumber,
        /// Block number when payout was executed
        executed_block: BlockNumber,
    }

    /// Defines the storage of your treasury contract.
    #[ink(storage)]
    pub struct Treasury {
        /// Owner of the treasury
        owner: AccountId,
        /// List of treasurers who can add payouts
        treasurers: Vec<AccountId>,
        /// Pending payouts
        pending_payouts: Vec<PendingPayout>,
        /// Payment schedules
        payout_schedules: Vec<PayoutSchedule>,
        /// List of registered asset ids
        registered_assets: Vec<TokenId>,
        cutoff_blocks: BlockNumber, // Number of blocks to keep in active storage
        past_payouts: Vec<PastPayout>,
        /// Thresholds for treasurer approvals
        thresholds: Vec<Threshold>,
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
                treasurers: Vec::new(),
                pending_payouts: Vec::new(),
                payout_schedules: Vec::new(),
                registered_assets: Vec::new(),
                cutoff_blocks: 432_000, // 30 days (1 block = 6 seconds)
                past_payouts: Vec::new(),
                thresholds,
            })
        }

        /// Add a new treasurer
        #[ink(message)]
        pub fn add_treasurer(&mut self, treasurer: AccountId) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }

            if self.treasurers.contains(&treasurer) {
                #[cfg(feature = "std")]
                ink::env::debug_println!(
                    "Treasurer already exists: {}",
                    format_account_id(&treasurer)
                );
                return Err(Error::TreasurerExists);
            }

            self.treasurers.push(treasurer);
            self.env().emit_event(TreasurerAdded { treasurer });

            Ok(())
        }

        /// Remove a treasurer
        #[ink(message)]
        pub fn remove_treasurer(&mut self, treasurer: AccountId) -> Result<()> {
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
        pub fn add_payout(&mut self, to: AccountId, amount: Balance) -> Result<u32> {
            #[cfg(feature = "std")]
            ink::env::debug_println!("Adding payout: {} to {}", amount, format_account_id(&to));

            if !self.treasurers.contains(&self.env().caller()) {
                #[cfg(feature = "std")]
                ink::env::debug_println!(
                    "Caller is not a treasurer: {}",
                    format_account_id(&self.env().caller())
                );
                return Err(Error::NotTreasurer);
            }

            let balance = self.env().balance();
            if balance < amount {
                ink::env::debug_println!("Insufficient balance: {} < {}", balance, amount);
                return Err(Error::InsufficientBalance);
            }

            // Clean up past payouts based on cutoff
            let cutoff_block = self.get_cutoff_block();
            self.past_payouts
                .retain(|payout| payout.executed_block > cutoff_block);

            // Generate new payout ID
            let payout_id = u32::try_from(self.pending_payouts.len())
                .map(|len| len.saturating_add(1))
                .unwrap_or(1);

            // Add new payout with initial approval
            let mut approvals = Vec::new();
            approvals.push(self.env().caller());
            self.pending_payouts.push(PendingPayout {
                id: payout_id,
                to,
                amount,
                block_number: self.env().block_number(),
                approvals,
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
                    ink::env::debug_println!("Approved payout: {}", payout_id);
                    payout.approvals.push(caller);
                }
                return Ok(());
            }

            Err(Error::PayoutNotFound)
        }

        /// Process pending payouts
        #[ink(message)]
        pub fn process_pending_payouts(&mut self) -> Result<()> {
            ink::env::debug_println!("Processing {} pending payouts", self.pending_payouts.len());
            let current_block = self.env().block_number();

            let mut aggregated_payouts: BTreeMap<AccountId, Balance> = BTreeMap::new();
            let mut individual_payouts: BTreeMap<AccountId, Vec<PendingPayout>> = BTreeMap::new();

            // First aggregate all payouts and group individual payouts
            for payout in &self.pending_payouts {
                // Get required approvals for this amount
                let required_approvals = self
                    .thresholds
                    .iter()
                    .find(|t| payout.amount >= t.min_amount && payout.amount <= t.max_amount)
                    .map(|t| t.required_approvals)
                    .unwrap_or(1)
                    .min(u32::try_from(self.treasurers.len()).unwrap_or(u32::MAX));

                ink::env::debug_println!(
                    "Processing payout: {} with {} approvals, required {}",
                    payout.id,
                    payout.approvals.len(),
                    required_approvals
                );

                // Skip if not enough approvals
                if payout.approvals.len() < required_approvals as usize {
                    continue;
                }

                let current = aggregated_payouts.get(&payout.to).copied().unwrap_or(0);
                aggregated_payouts.insert(payout.to, current.saturating_add(payout.amount));

                individual_payouts
                    .entry(payout.to)
                    .or_insert_with(Vec::new)
                    .push(payout.clone());
            }

            let mut total_amount: Balance = 0;
            let mut payouts_count: u32 = 0;

            // Process each aggregated payout
            for (to, amount) in aggregated_payouts {
                #[cfg(feature = "std")]
                ink::env::debug_println!(
                    "Processing payout: {} to {}",
                    amount,
                    format_account_id(&to)
                );
                if self.env().transfer(to, amount).is_err() {
                    #[cfg(feature = "std")]
                    ink::env::debug_println!("Transfer failed for {}", format_account_id(&to));
                } else {
                    total_amount = total_amount.saturating_add(amount);
                    payouts_count = payouts_count.saturating_add(1);

                    // Add individual payouts to past_payouts
                    if let Some(payouts) = individual_payouts.get(&to) {
                        for payout in payouts {
                            self.past_payouts.push(PastPayout {
                                to,
                                amount: payout.amount,
                                asset_id: 0, // Native token
                                payout_block: payout.block_number,
                                executed_block: current_block,
                            });
                        }
                    }
                }
            }

            // Clear pending payouts that were processed
            self.pending_payouts.retain(|p| {
                let required_approvals = self
                    .thresholds
                    .iter()
                    .find(|t| p.amount >= t.min_amount && p.amount <= t.max_amount)
                    .map(|t| t.required_approvals)
                    .unwrap_or(1)
                    .min(u32::try_from(self.treasurers.len()).unwrap_or(u32::MAX));
                p.approvals.len() < required_approvals as usize
            });

            ink::env::debug_println!(
                "Processed {} payouts, total amount: {}",
                payouts_count,
                total_amount
            );

            self.env().emit_event(PayoutsProcessed {
                total_amount,
                payouts_count,
            });

            Ok(())
        }

        /// Register a new asset id the contract tracks and handles
        #[ink(message)]
        pub fn register_asset_id(&mut self, asset_id: TokenId) -> Result<()> {
            ink::env::debug_println!("Registering asset id: {}", asset_id);
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            if self.registered_assets.contains(&asset_id) {
                return Ok(()); // already registered, no-op
            }
            ink::env::debug_println!("Registering asset id: {}", asset_id);
            self.registered_assets.push(asset_id);
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
            self.treasurers.clone()
        }

        /// Get the pending payouts
        #[ink(message)]
        pub fn get_pending_payouts(&self) -> Vec<PendingPayout> {
            self.pending_payouts.clone()
        }

        /// Create a new scheduled payout
        #[ink(message)]
        pub fn create_scheduled_payout(
            &mut self,
            recipient: AccountId,
            amount: Balance,
            start_block: BlockNumber,
        ) -> Result<()> {
            // Stub implementation
            Ok(())
        }

        /// Create a new recurring payout
        #[ink(message)]
        pub fn create_recurring_payout(
            &mut self,
            recipient: AccountId,
            amount: Balance,
            start_block: BlockNumber,
            interval_blocks: BlockNumber,
            end_block: Option<BlockNumber>,
        ) -> Result<()> {
            // Stub implementation
            Ok(())
        }

        /// Create a new vested payout
        #[ink(message)]
        pub fn create_vested_payout(
            &mut self,
            recipient: AccountId,
            total_amount: Balance,
            start_block: BlockNumber,
            vesting_period_blocks: BlockNumber,
            cliff_blocks: Option<BlockNumber>,
        ) -> Result<()> {
            // Stub implementation
            Ok(())
        }

        /// Cancel a payout
        #[ink(message)]
        pub fn cancel_payout(&mut self, payout_id: u32) -> Result<()> {
            // Stub implementation
            Ok(())
        }

        /// Get a payout schedule
        #[ink(message)]
        pub fn get_payout_schedule(&self, payout_id: u32) -> Option<PayoutSchedule> {
            // Stub implementation
            None
        }

        /// Get active payouts
        #[ink(message)]
        pub fn get_active_payouts(&self) -> Vec<PayoutSchedule> {
            // Stub implementation
            Vec::new()
        }

        /// Get completed payouts
        #[ink(message)]
        pub fn get_completed_payouts(&self) -> Vec<PayoutSchedule> {
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

        /// Helper function to calculate cutoff block
        fn get_cutoff_block(&self) -> BlockNumber {
            self.env().block_number().saturating_sub(self.cutoff_blocks)
        }

        #[ink(message)]
        pub fn get_cutoff_blocks(&self) -> BlockNumber {
            self.cutoff_blocks
        }

        #[ink(message)]
        pub fn set_cutoff_blocks(&mut self, new_cutoff: BlockNumber) -> Result<()> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            self.cutoff_blocks = new_cutoff;
            Ok(())
        }

        #[ink(message)]
        pub fn get_past_payouts(&self) -> Vec<PastPayout> {
            self.past_payouts.clone()
        }

        #[ink(message)]
        pub fn get_payouts_by_block_range(
            &self,
            start_block: BlockNumber,
            end_block: BlockNumber,
        ) -> Vec<PayoutSchedule> {
            let mut result = Vec::new();

            // Add payouts from past_payouts
            for payout in &self.past_payouts {
                if payout.payout_block >= start_block && payout.executed_block <= end_block {
                    result.push(PayoutSchedule {
                        recipient: payout.to,
                        amount: payout.amount,
                        start_block: payout.payout_block,
                        end_block: payout.executed_block,
                        payout_type: PayoutType::Scheduled,
                        status: PayoutStatus::Completed,
                    });
                }
            }

            // Add payouts from current schedules
            for payout in &self.payout_schedules {
                if payout.start_block >= start_block && payout.end_block <= end_block {
                    result.push(payout.clone());
                }
            }

            result
        }

        // New message to get current cutoff block
        #[ink(message)]
        pub fn get_current_cutoff_block(&self) -> BlockNumber {
            self.get_cutoff_block()
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
        use ink::env::DefaultEnvironment;

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

            // Set initial block number
            let initial_block = 500_000;
            test::set_block_number::<DefaultEnvironment>(initial_block);

            // Set caller as treasurer (bob)
            test::set_caller::<DefaultEnvironment>(accounts.bob);

            // Add some payouts in the future
            treasury.add_payout(accounts.charlie, 100).unwrap(); // payout1
            treasury.add_payout(accounts.django, 200).unwrap(); // payout2
            treasury.add_payout(accounts.eve, 300).unwrap(); // payout3

            // Get the payouts from pending_payouts
            let payouts = treasury.get_pending_payouts();
            assert_eq!(payouts.len(), 3);

            // Progress blocks to just before payout1's start
            test::set_block_number::<DefaultEnvironment>(initial_block + 99);

            // Process payouts
            treasury.process_pending_payouts().unwrap();

            // Verify payouts were moved from pending to past
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            let past_payouts = treasury.get_past_payouts();
            assert_eq!(past_payouts.len(), 3);

            // Verify the amounts in past_payouts
            let total_amount = past_payouts.iter().map(|p| p.amount).sum::<Balance>();
            assert_eq!(total_amount, 600); // 100 + 200 + 300

            // Progress blocks to after payout1's end
            test::set_block_number::<DefaultEnvironment>(initial_block + 151);

            // Add another payout to trigger cleanup
            treasury.add_payout(accounts.charlie, 400).unwrap();

            // Progress blocks to after payout2's end
            test::set_block_number::<DefaultEnvironment>(initial_block + 432_051);

            // Add another payout to trigger cleanup
            treasury.add_payout(accounts.charlie, 500).unwrap();

            // Test with 1-day cutoff
            test::set_caller::<DefaultEnvironment>(accounts.alice); // owner
            treasury.set_cutoff_blocks(14_400).unwrap(); // 1 day

            // Progress blocks to after payout3's end
            test::set_block_number::<DefaultEnvironment>(initial_block + 432_151);

            // Set caller as treasurer and add a new payout to trigger cleanup
            test::set_caller::<DefaultEnvironment>(accounts.bob);
            treasury.add_payout(accounts.charlie, 600).unwrap();

            // Verify past_payouts cleanup with 1-day cutoff
            // All payouts should be removed as they're all older than 1 day
            assert!(treasury.get_past_payouts().is_empty());
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

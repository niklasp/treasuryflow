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

    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PayoutSchedule {
        pub recipient: AccountId,
        pub amount: Balance,
        pub start_block: BlockNumber,
        pub end_block: BlockNumber,
        pub payout_type: PayoutType,
        pub status: PayoutStatus,
    }

    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PayoutType {
        Scheduled = 0,
        Recurring = 1,
        Vested = 2,
    }

    #[derive(Debug, Encode, Decode, Clone)]
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
        /// Recipient of the payout
        to: AccountId,
        /// Amount to be paid out
        amount: Balance,
        /// Block number when payout was added
        block_number: BlockNumber,
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
            Ok(Self {
                owner: Self::env().caller(),
                treasurers: Vec::new(),
                pending_payouts: Vec::new(),
                payout_schedules: Vec::new(),
                registered_assets: Vec::new(),
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
        pub fn add_payout(&mut self, to: AccountId, amount: Balance) -> Result<()> {
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
            ink::env::debug_println!("Processing pending payouts");
            let current_block = self.env().block_number();

            let mut aggregated_payouts: BTreeMap<AccountId, Balance> = BTreeMap::new();

            for payout in &self.pending_payouts {
                let current = aggregated_payouts.get(&payout.to).copied().unwrap_or(0);
                aggregated_payouts.insert(payout.to, current.saturating_add(payout.amount));
            }

            let mut total_amount: Balance = 0;
            let mut payouts_count: u32 = 0;

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
                }
            }

            self.pending_payouts.clear();

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

        /// Process payouts
        #[ink(message)]
        pub fn process_payouts(&mut self) -> Result<()> {
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
    }

    /// Unit tests for the treasury contract
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;
        use ink::env::DefaultEnvironment;

        const INITIAL_BALANCE: Balance = 1000000000;

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
            assert_eq!(treasury.get_pending_payouts().len(), 0);
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
    }
}

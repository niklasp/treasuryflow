#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod treasury {
    use ink::prelude::vec::Vec;
    use ink::storage::{Mapping, StorageVec};
    use ink::{H160, U256};
    use parity_scale_codec::{Decode, Encode};

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum PayoutStatus {
        Pending,
        Active,
        Completed(u32), // block number when completed
        Cancelled(u32), // block number when cancelled
    }

    // These structs hold the fundamental, user-provided data for each payout type.
    // They are the "source of truth" for a payout's configuration.

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct OneTimeData {
        pub to: H160,
        pub amount: U256,
        pub scheduled_block: Option<u32>,
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct RecurringData {
        pub to: H160,
        pub amount_per_payment: U256,
        pub start_block: Option<u32>,
        pub interval_blocks: u32,
        pub total_payments: u32,
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct VestedData {
        pub to: H160,
        pub total_amount: U256,
        pub cliff_block: Option<u32>,
        pub vesting_duration_blocks: u32,
        pub vesting_interval_blocks: u32,
    }

    /// Input specification for creating new payouts.
    /// Each variant holds the fundamental data for that payout type.
    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PayoutRequest {
        OneTime(OneTimeData),
        Recurring(RecurringData),
        Vested(VestedData),
    }

    // These structs represent the fully stored payout objects, combining the
    // original data with contract-managed state.

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct StoredOneTimePayout {
        pub data: OneTimeData, // Composed data
        // --- Contract-managed state ---
        pub id: u32,
        pub status: PayoutStatus,
        pub created_block: u32,
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct StoredRecurringPayout {
        pub data: RecurringData, // Composed data
        // --- Contract-managed state ---
        pub id: u32,
        pub remaining_payments: u32,
        pub status: PayoutStatus,
        pub created_block: u32,
    }

    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct StoredVestedPayout {
        pub data: VestedData, // Composed data
        // --- Contract-managed state ---
        pub id: u32,
        pub remaining_periods: u32,
        pub original_total_periods: u32,
        pub released_amount: U256,
        pub status: PayoutStatus,
        pub created_block: u32,
    }

    /// The actual payout object managed by the contract.
    /// Each variant wraps a stored object that combines original data with state.
    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Payout {
        OneTime(StoredOneTimePayout),
        Recurring(StoredRecurringPayout),
        Vested(StoredVestedPayout),
    }

    #[ink(storage)]
    pub struct Treasury {
        owner: H160,
        pending_payout_ids: Vec<u32>,
        payouts: StorageVec<Payout>,            // All pending payouts
        processed_payout_ids: Vec<u32>,         // Complete list of all processed payout IDs
        archived_payouts: Mapping<u32, Payout>, // All processed payouts, queryable by ID
        is_processing: bool,
        next_payout_id: u32,
    }

    /// Events emitted by the treasury contract
    #[derive(Debug, Encode, Decode, Clone, PartialEq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PayoutType {
        OneTime = 0,
        Recurring = 1,
        Vested = 2,
    }

    #[ink(event)]
    pub struct TreasuryCreated {
        #[ink(topic)]
        owner: H160,
    }

    #[ink(event)]
    pub struct PayoutAdded {
        #[ink(topic)]
        payout_id: u32,
        #[ink(topic)]
        to: H160,
        #[ink(topic)]
        payout_type: PayoutType,
        amount: U256,
        payout_data: Payout,
    }

    #[ink(event)]
    pub struct PayoutsProcessed {
        processed_ids: Vec<u32>,
        total_amount: U256,
    }

    #[ink(event)]
    pub struct FundsAdded {
        #[ink(topic)]
        from: H160,
        amount: U256,
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
        /// Reentrancy detected
        Reentrancy = 7,
        /// Precision loss (amount is not divisible by PRECISION_FACTOR)
        PrecisionLoss = 8,
        /// Invalid cliff block (must be in the future)
        InvalidCliffBlock = 9,
        /// Invalid vesting duration (must be greater than 0)
        InvalidVestingDuration = 10,
        /// Invalid vesting interval (must be greater than 0)
        InvalidVestingInterval = 11,
    }

    impl Treasury {
        #[ink(constructor)]
        pub fn new() -> Self {
            let instance = Self {
                owner: ink::env::caller(),
                pending_payout_ids: Vec::new(),
                payouts: StorageVec::new(),
                processed_payout_ids: Vec::new(),
                archived_payouts: Mapping::new(),
                is_processing: false,
                next_payout_id: 0,
            };

            Self::env().emit_event(TreasuryCreated {
                owner: instance.owner,
            });

            instance
        }

        /// Helper function to check if a payout is ready to be processed
        fn is_ready(&self, payout: &Payout) -> bool {
            let current_block = self.env().block_number();

            match payout {
                Payout::OneTime(stored) => {
                    match stored.data.scheduled_block {
                        None => true, // Immediate payout
                        Some(block) => current_block >= block,
                    }
                }
                Payout::Recurring(stored) => {
                    // Check if it's time for this recurring payment
                    match stored.data.start_block {
                        None => true, // Start immediately
                        Some(start_block) => current_block >= start_block,
                    }
                }
                Payout::Vested(stored) => {
                    // Check if cliff period has passed
                    match stored.data.cliff_block {
                        None => true, // Start vesting immediately
                        Some(cliff_block) => current_block >= cliff_block,
                    }
                }
            }
        }

        /// Move a processed payout to history
        fn move_to_processed(&mut self, mut payout: Payout) {
            // Update status to completed with current block number
            match &mut payout {
                Payout::OneTime(stored) => {
                    stored.status = PayoutStatus::Completed(self.env().block_number());
                }
                Payout::Recurring(stored) => {
                    stored.status = PayoutStatus::Completed(self.env().block_number());
                }
                Payout::Vested(stored) => {
                    stored.status = PayoutStatus::Completed(self.env().block_number());
                }
            }

            // Store in archived payouts (always accessible by ID)
            let payout_id = match &payout {
                Payout::OneTime(stored) => stored.id,
                Payout::Recurring(stored) => stored.id,
                Payout::Vested(stored) => stored.id,
            };
            self.archived_payouts.insert(payout_id, &payout);

            // Add to complete processed IDs list (no limit)
            self.processed_payout_ids.push(payout_id);
        }

        /// Helper function to validate amount is large enough to avoid precision loss
        fn is_valid_precision_amount(amount: U256) -> bool {
            const PRECISION_FACTOR: U256 = U256([1_000_000, 0, 0, 0]); // 1e6

            // Minimum amount check: must be at least PRECISION_FACTOR
            if amount < PRECISION_FACTOR {
                return false;
            }

            // Safe divisibility check: divide and multiply back, compare with original
            // This avoids modulo operation entirely
            let divided = amount.checked_div(PRECISION_FACTOR).unwrap_or(U256::zero());
            let multiplied_back = divided
                .checked_mul(PRECISION_FACTOR)
                .unwrap_or(U256::zero());

            // If amount is divisible by PRECISION_FACTOR, then divided * PRECISION_FACTOR == amount
            amount == multiplied_back
        }

        #[ink(message)]
        pub fn get_processing(&self) -> bool {
            self.is_processing
        }

        #[ink(message)]
        pub fn get_pending_payouts(&self) -> Vec<Payout> {
            self.pending_payout_ids
                .iter()
                .filter_map(|&id| {
                    // Find payout by ID in the storage vec
                    for i in 0..self.payouts.len() {
                        if let Some(payout) = self.payouts.get(i) {
                            let payout_id = match &payout {
                                Payout::OneTime(stored) => stored.id,
                                Payout::Recurring(stored) => stored.id,
                                Payout::Vested(stored) => stored.id,
                            };
                            if payout_id == id {
                                // Only return payouts with Pending status
                                let status = match &payout {
                                    Payout::OneTime(stored) => &stored.status,
                                    Payout::Recurring(stored) => &stored.status,
                                    Payout::Vested(stored) => &stored.status,
                                };
                                match status {
                                    PayoutStatus::Pending => return Some(payout),
                                    _ => return None,
                                }
                            }
                        }
                    }
                    None
                })
                .collect()
        }

        #[ink(message)]
        pub fn get_ready_payouts(&self) -> Vec<Payout> {
            self.get_pending_payouts()
                .into_iter()
                .filter(|payout| self.is_ready(payout))
                .collect()
        }

        #[ink(message)]
        pub fn get_scheduled_payouts(&self) -> Vec<Payout> {
            self.get_pending_payouts()
                .into_iter()
                .filter(|payout| !self.is_ready(payout))
                .collect()
        }

        #[ink(message)]
        pub fn get_recurring_payouts(&self) -> Vec<Payout> {
            self.get_pending_payouts()
                .into_iter()
                .filter(|payout| matches!(payout, Payout::Recurring(_)))
                .collect()
        }

        #[ink(message)]
        pub fn get_vested_payouts(&self) -> Vec<Payout> {
            self.get_pending_payouts()
                .into_iter()
                .filter(|payout| matches!(payout, Payout::Vested(_)))
                .collect()
        }

        #[ink(message)]
        pub fn get_processed_payout_ids(&self) -> Vec<u32> {
            self.processed_payout_ids.clone()
        }

        #[ink(message)]
        pub fn get_recent_processed_payouts(&self, count: u32) -> Vec<Payout> {
            let count = count as usize;
            let total_processed = self.processed_payout_ids.len();

            if total_processed == 0 {
                return Vec::new();
            }

            // Get the most recent IDs (from the end of the vector)
            let start_index = total_processed.saturating_sub(count);

            let recent_ids = &self.processed_payout_ids[start_index..];

            // Retrieve full payout data for these IDs
            recent_ids
                .iter()
                .rev() // Most recent first
                .filter_map(|&id| self.archived_payouts.get(id))
                .collect()
        }

        #[ink(message)]
        pub fn get_payout(&self, id: u32) -> Option<Payout> {
            // First check archived (processed) payouts
            if let Some(payout) = self.archived_payouts.get(id) {
                return Some(payout);
            }

            // Then check pending payouts
            for i in 0..self.payouts.len() {
                if let Some(payout) = self.payouts.get(i) {
                    let payout_id = match &payout {
                        Payout::OneTime(stored) => stored.id,
                        Payout::Recurring(stored) => stored.id,
                        Payout::Vested(stored) => stored.id,
                    };
                    if payout_id == id {
                        return Some(payout);
                    }
                }
            }

            None
        }

        #[ink(message)]
        pub fn get_pending_payout_ids(&self) -> Vec<u32> {
            self.pending_payout_ids.clone()
        }

        #[ink(message)]
        pub fn get_balance(&self) -> U256 {
            self.env().balance()
        }

        #[ink(message, payable)]
        pub fn fund(&mut self) -> Result<U256, Error> {
            let transferred_value = self.env().transferred_value();
            let caller = self.env().caller();

            // Convert AccountId to H160 for the event (take first 20 bytes)
            let mut caller_bytes = [0u8; 20];
            let caller_ref = caller.as_ref();
            let copy_len = caller_ref.len().min(20);
            caller_bytes[..copy_len].copy_from_slice(&caller_ref[..copy_len]);
            let caller_h160 = H160::from(caller_bytes);

            self.env().emit_event(FundsAdded {
                from: caller_h160,
                amount: transferred_value,
            });

            Ok(transferred_value)
        }

        /// Internal function to handle common payout storage logic
        fn add_payout_internal(&mut self, payout: Payout) -> Result<u32, Error> {
            if self.env().caller() != self.owner {
                return Err(Error::NotOwner);
            }
            let payout_id = match &payout {
                Payout::OneTime(stored) => stored.id,
                Payout::Recurring(stored) => stored.id,
                Payout::Vested(stored) => stored.id,
            };

            let (to, amount, payout_type) = match &payout {
                Payout::OneTime(stored) => {
                    (stored.data.to, stored.data.amount, PayoutType::OneTime)
                }
                Payout::Recurring(stored) => (
                    stored.data.to,
                    stored.data.amount_per_payment,
                    PayoutType::Recurring,
                ),
                Payout::Vested(stored) => {
                    (stored.data.to, stored.data.total_amount, PayoutType::Vested)
                }
            };

            // Validate amount for precision safety
            if !Self::is_valid_precision_amount(amount) {
                return Err(Error::PrecisionLoss);
            }

            self.payouts.push(&payout);
            self.pending_payout_ids.push(payout_id);

            self.env().emit_event(PayoutAdded {
                payout_id,
                to,
                payout_type,
                amount,
                payout_data: payout.clone(),
            });

            self.next_payout_id = self.next_payout_id.saturating_add(1);
            Ok(payout_id)
        }

        #[ink(message)]
        pub fn add_payout(
            &mut self,
            to: H160,
            amount: U256,
            scheduled_block: Option<u32>,
        ) -> Result<u32, Error> {
            let id = self.next_payout_id;
            let payout = Payout::OneTime(StoredOneTimePayout {
                data: OneTimeData {
                    to,
                    amount,
                    scheduled_block,
                },
                id,
                status: PayoutStatus::Pending,
                created_block: self.env().block_number(),
            });

            self.add_payout_internal(payout)
        }

        #[ink(message)]
        pub fn add_recurring_payout(
            &mut self,
            to: H160,
            amount_per_payment: U256,
            start_block: Option<u32>,
            interval_blocks: u32,
            total_payments: u32,
        ) -> Result<u32, Error> {
            let id = self.next_payout_id;
            let payout = Payout::Recurring(StoredRecurringPayout {
                data: RecurringData {
                    to,
                    amount_per_payment,
                    start_block,
                    interval_blocks,
                    total_payments,
                },
                id,
                remaining_payments: total_payments,
                status: PayoutStatus::Pending,
                created_block: self.env().block_number(),
            });

            self.add_payout_internal(payout)
        }

        #[ink(message)]
        pub fn add_vested_payout(
            &mut self,
            to: H160,
            total_amount: U256,
            cliff_block: Option<u32>,
            vesting_duration_blocks: u32,
            vesting_interval_blocks: u32,
        ) -> Result<u32, Error> {
            // Calculate amount per vesting period
            let total_periods = vesting_duration_blocks
                .checked_div(vesting_interval_blocks)
                .unwrap_or(0);
            if total_periods == 0 {
                return Err(Error::InvalidFrequency);
            }
            let _amount_per_period = total_amount
                .checked_div(U256::from(total_periods))
                .unwrap_or(U256::zero());

            let id = self.next_payout_id;
            let payout = Payout::Vested(StoredVestedPayout {
                data: VestedData {
                    to,
                    total_amount,
                    cliff_block,
                    vesting_duration_blocks,
                    vesting_interval_blocks,
                },
                id,
                remaining_periods: total_periods,
                original_total_periods: total_periods,
                released_amount: U256::from(0),
                status: PayoutStatus::Pending,
                created_block: self.env().block_number(),
            });

            self.add_payout_internal(payout)
        }

        #[ink(message)]
        pub fn add_payouts(&mut self, payouts: Vec<PayoutRequest>) -> Result<Vec<u32>, Error> {
            let mut payout_ids = Vec::new();

            // Validate all payouts first (all-or-nothing approach)
            for payout_def in &payouts {
                match payout_def {
                    PayoutRequest::OneTime(data) => {
                        if !Self::is_valid_precision_amount(data.amount) {
                            return Err(Error::PrecisionLoss);
                        }
                    }
                    PayoutRequest::Recurring(data) => {
                        if !Self::is_valid_precision_amount(data.amount_per_payment) {
                            return Err(Error::PrecisionLoss);
                        }
                    }
                    PayoutRequest::Vested(data) => {
                        if !Self::is_valid_precision_amount(data.total_amount) {
                            return Err(Error::PrecisionLoss);
                        }
                        let total_periods = data
                            .vesting_duration_blocks
                            .checked_div(data.vesting_interval_blocks)
                            .unwrap_or(0);
                        if total_periods == 0 {
                            return Err(Error::InvalidFrequency);
                        }
                    }
                }
            }

            // If all validations pass, create all payouts
            for payout_def in payouts {
                let id = match payout_def {
                    PayoutRequest::OneTime(data) => {
                        self.add_payout(data.to, data.amount, data.scheduled_block)?
                    }
                    PayoutRequest::Recurring(data) => self.add_recurring_payout(
                        data.to,
                        data.amount_per_payment,
                        data.start_block,
                        data.interval_blocks,
                        data.total_payments,
                    )?,
                    PayoutRequest::Vested(data) => self.add_vested_payout(
                        data.to,
                        data.total_amount,
                        data.cliff_block,
                        data.vesting_duration_blocks,
                        data.vesting_interval_blocks,
                    )?,
                };
                payout_ids.push(id);
            }

            Ok(payout_ids)
        }

        #[ink(message)]
        pub fn cancel_payout(&mut self, payout_id: u32) -> Result<(), Error> {
            // First check if the payout is actually in pending list
            if !self.pending_payout_ids.contains(&payout_id) {
                return Err(Error::PayoutNotFound);
            }

            // Find the payout in storage (only allow cancelling pending payouts)
            let mut payout_found = None;
            for i in 0..self.payouts.len() {
                if let Some(payout) = self.payouts.get(i) {
                    let payout_id_match = match &payout {
                        Payout::OneTime(stored) => stored.id,
                        Payout::Recurring(stored) => stored.id,
                        Payout::Vested(stored) => stored.id,
                    };
                    let status_pending = match &payout {
                        Payout::OneTime(stored) => matches!(stored.status, PayoutStatus::Pending),
                        Payout::Recurring(stored) => matches!(stored.status, PayoutStatus::Pending),
                        Payout::Vested(stored) => matches!(stored.status, PayoutStatus::Pending),
                    };
                    if payout_id_match == payout_id && status_pending {
                        payout_found = Some(payout.clone());
                        break;
                    }
                }
            }

            if let Some(mut payout) = payout_found {
                // Update status to cancelled with current block number
                match &mut payout {
                    Payout::OneTime(stored) => {
                        stored.status = PayoutStatus::Cancelled(self.env().block_number());
                    }
                    Payout::Recurring(stored) => {
                        stored.status = PayoutStatus::Cancelled(self.env().block_number());
                    }
                    Payout::Vested(stored) => {
                        stored.status = PayoutStatus::Cancelled(self.env().block_number());
                    }
                }

                // Move to archived payouts (same as processed, but cancelled)
                let payout_id = match &payout {
                    Payout::OneTime(stored) => stored.id,
                    Payout::Recurring(stored) => stored.id,
                    Payout::Vested(stored) => stored.id,
                };
                self.archived_payouts.insert(payout_id, &payout);
                self.processed_payout_ids.push(payout_id);

                // Remove from pending payouts
                self.pending_payout_ids.retain(|&id| id != payout_id);

                Ok(())
            } else {
                Err(Error::PayoutNotFound)
            }
        }

        #[ink(message)]
        pub fn process_payouts(&mut self) -> Result<(Vec<u32>, U256), Error> {
            // Reentrancy guard
            if self.is_processing {
                return Err(Error::Reentrancy);
            }
            self.is_processing = true;

            let mut ready_payouts = Vec::new();
            let mut total_amount = U256::from(0);

            // Find ready payouts (only those that are ready to be processed and have Pending status)
            let pending_ids = self.pending_payout_ids.clone();
            for payout_id in pending_ids.iter() {
                // Find the payout by ID
                for i in 0..self.payouts.len() {
                    if let Some(payout) = self.payouts.get(i) {
                        let payout_id_match = match &payout {
                            Payout::OneTime(stored) => stored.id,
                            Payout::Recurring(stored) => stored.id,
                            Payout::Vested(stored) => stored.id,
                        };
                        let status_pending = match &payout {
                            Payout::OneTime(stored) => {
                                matches!(stored.status, PayoutStatus::Pending)
                            }
                            Payout::Recurring(stored) => {
                                matches!(stored.status, PayoutStatus::Pending)
                            }
                            Payout::Vested(stored) => {
                                matches!(stored.status, PayoutStatus::Pending)
                            }
                        };
                        if payout_id_match == *payout_id && self.is_ready(&payout) && status_pending
                        {
                            let amount = match &payout {
                                Payout::OneTime(stored) => stored.data.amount,
                                Payout::Recurring(stored) => stored.data.amount_per_payment,
                                Payout::Vested(stored) => {
                                    // Calculate current vesting payment amount
                                    if stored.remaining_periods == 1 {
                                        // Final payment: pay the remainder
                                        stored
                                            .data
                                            .total_amount
                                            .saturating_sub(stored.released_amount)
                                    } else {
                                        // Regular payment: divide by original total periods
                                        stored
                                            .data
                                            .total_amount
                                            .checked_div(U256::from(stored.original_total_periods))
                                            .unwrap_or(U256::zero())
                                    }
                                }
                            };
                            ready_payouts.push(payout.clone());
                            total_amount = total_amount.saturating_add(amount);
                            break;
                        }
                    }
                }
            }

            // Process only the ready payouts
            let mut processed_ids = Vec::new();
            for payout in ready_payouts.iter() {
                let (to, amount) = match payout {
                    Payout::OneTime(stored) => (stored.data.to, stored.data.amount),
                    Payout::Recurring(stored) => (stored.data.to, stored.data.amount_per_payment),
                    Payout::Vested(stored) => {
                        let amount = if stored.remaining_periods == 1 {
                            // Final payment: pay the remainder
                            stored
                                .data
                                .total_amount
                                .saturating_sub(stored.released_amount)
                        } else {
                            // Regular payment: divide by original total periods
                            stored
                                .data
                                .total_amount
                                .checked_div(U256::from(stored.original_total_periods))
                                .unwrap_or(U256::zero())
                        };
                        (stored.data.to, amount)
                    }
                };
                let transfer_result = self.env().transfer(to, amount);
                if transfer_result.is_err() {
                    self.is_processing = false;
                    return Err(Error::InsufficientBalance);
                }
                let payout_id = match payout {
                    Payout::OneTime(stored) => stored.id,
                    Payout::Recurring(stored) => stored.id,
                    Payout::Vested(stored) => stored.id,
                };
                processed_ids.push(payout_id);
            }

            // Move processed payouts to history and handle follow-up payouts
            for payout in ready_payouts {
                // Handle follow-up payouts for recurring and vested types
                match &payout {
                    Payout::OneTime(_stored) => {
                        // OneTime payouts are just completed
                    }
                    Payout::Recurring(stored) => {
                        // Create next recurring payment if more payments remain
                        if stored.remaining_payments > 1 {
                            let next_payout = Payout::Recurring(StoredRecurringPayout {
                                data: RecurringData {
                                    to: stored.data.to,
                                    amount_per_payment: stored.data.amount_per_payment,
                                    start_block: Some(
                                        self.env()
                                            .block_number()
                                            .saturating_add(stored.data.interval_blocks),
                                    ),
                                    interval_blocks: stored.data.interval_blocks,
                                    total_payments: stored.remaining_payments.saturating_sub(1),
                                },
                                id: self.next_payout_id,
                                remaining_payments: stored.remaining_payments.saturating_sub(1),
                                status: PayoutStatus::Pending,
                                created_block: stored.created_block,
                            });

                            self.payouts.push(&next_payout);
                            self.pending_payout_ids.push(self.next_payout_id);
                            self.next_payout_id = self.next_payout_id.saturating_add(1);
                        }
                    }
                    Payout::Vested(stored) => {
                        // Calculate the amount that was just paid
                        let current_payment_amount = if stored.remaining_periods == 1 {
                            // Final payment: pay the remainder
                            stored
                                .data
                                .total_amount
                                .saturating_sub(stored.released_amount)
                        } else {
                            // Regular payment: divide by original total periods
                            stored
                                .data
                                .total_amount
                                .checked_div(U256::from(stored.original_total_periods))
                                .unwrap_or(U256::zero())
                        };
                        let new_released_amount = stored
                            .released_amount
                            .saturating_add(current_payment_amount);

                        // Check if more vesting periods remain and total hasn't been exceeded
                        if stored.remaining_periods > 1
                            && new_released_amount < stored.data.total_amount
                        {
                            let next_vesting_block = self
                                .env()
                                .block_number()
                                .saturating_add(stored.data.vesting_interval_blocks);

                            // For follow-up vested payouts, total_amount should be what's left to pay
                            let _remaining_amount =
                                stored.data.total_amount.saturating_sub(new_released_amount);

                            let next_payout = Payout::Vested(StoredVestedPayout {
                                data: VestedData {
                                    to: stored.data.to,
                                    total_amount: stored.data.total_amount, // Keep original total for consistent calculation
                                    cliff_block: Some(next_vesting_block),
                                    vesting_duration_blocks: stored.data.vesting_duration_blocks,
                                    vesting_interval_blocks: stored.data.vesting_interval_blocks,
                                },
                                id: self.next_payout_id,
                                remaining_periods: stored.remaining_periods.saturating_sub(1),
                                original_total_periods: stored.original_total_periods, // Pass through original
                                released_amount: new_released_amount, // Track cumulative released amount
                                status: PayoutStatus::Pending,
                                created_block: stored.created_block, // Keep original creation block
                            });

                            self.payouts.push(&next_payout);
                            self.pending_payout_ids.push(self.next_payout_id);
                            self.next_payout_id = self.next_payout_id.saturating_add(1);
                        }
                    }
                }

                self.move_to_processed(payout);
            }

            // Remove only processed IDs from pending (leave scheduled ones that aren't ready)
            self.pending_payout_ids
                .retain(|id| !processed_ids.contains(id));

            // Emit event with processed IDs and total amount
            self.env().emit_event(PayoutsProcessed {
                processed_ids: processed_ids.clone(),
                total_amount,
            });

            self.is_processing = false;
            Ok((processed_ids, total_amount))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Test helper functions to reduce duplication
        fn setup_treasury_with_balance(balance: u128) -> Treasury {
            let owner = ink::env::caller();
            // Set the caller to be the owner for all treasury operations
            ink::env::test::set_caller(owner);
            let treasury = Treasury::new();
            let contract_address = ink::env::address();
            ink::env::test::set_account_balance(contract_address, U256::from(balance));
            treasury
        }

        fn setup_accounts() -> (H160, H160) {
            let recipient1 = ink::env::caller();
            let recipient2 = H160::from([1u8; 20]);
            (recipient1, recipient2)
        }

        fn add_and_process_payout(treasury: &mut Treasury, to: H160, amount: u128) -> u32 {
            let id = treasury.add_payout(to, U256::from(amount), None).unwrap();
            treasury.process_payouts().unwrap();
            id
        }

        fn create_multiple_processed_payouts(
            treasury: &mut Treasury,
            recipient: H160,
            count: u32,
        ) -> Vec<u32> {
            let mut ids = Vec::new();
            for i in 1..=count {
                let amount = i as u128 * 1_000_000; // 1e6, 2e6, 3e6, etc.
                let id = add_and_process_payout(treasury, recipient, amount);
                ids.push(id);
            }
            ids
        }

        #[ink::test]
        fn default_works() {
            let owner = ink::env::caller();
            // Set the caller to be the owner for all treasury operations
            ink::env::test::set_caller(owner);

            let treasury = Treasury::new();
            assert!(!treasury.get_processing());
        }

        #[ink::test]
        fn it_works() {
            let owner = ink::env::caller();
            // Set the caller to be the owner for all treasury operations
            ink::env::test::set_caller(owner);

            let mut treasury = Treasury::new();
            assert!(!treasury.get_processing());

            treasury
                .add_payout(ink::env::caller(), U256::from(1_000_000u128), None) // 1e6 - minimum amount
                .unwrap();
            assert!(treasury.get_pending_payouts().len() == 1);

            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids, vec![0]);
            assert_eq!(total_amount, U256::from(1_000_000u128));
            assert!(treasury.get_pending_payouts().len() == 0);
        }

        #[ink::test]
        fn test_add_100_payouts() {
            let owner = ink::env::caller();
            // Set the caller to be the owner for all treasury operations
            ink::env::test::set_caller(owner);

            let mut treasury = Treasury::new();
            let recipient = ink::env::caller();

            // Add 100 payouts
            for i in 0..100u32 {
                let amount = 1_000_000u128 + (i as u128 * 1_000_000u128); // Multiples of 1e6: 1e6, 2e6, 3e6, etc.
                let result = treasury.add_payout(recipient, U256::from(amount), None);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), i); // Check that IDs are sequential
            }

            // Verify all payouts were added
            assert_eq!(treasury.get_pending_payout_ids().len(), 100);
            assert_eq!(treasury.get_pending_payouts().len(), 100);

            // Verify the payouts have correct data
            let payouts = treasury.get_pending_payouts();
            for (index, payout) in payouts.iter().enumerate() {
                match payout {
                    Payout::OneTime(stored) => {
                        assert_eq!(stored.id, index as u32);
                        assert_eq!(stored.data.to, recipient);
                        assert_eq!(
                            stored.data.amount,
                            U256::from(1_000_000u128 + (index as u128 * 1_000_000u128))
                        );
                    }
                    _ => panic!("Expected OneTime payout"),
                }
            }

            // Verify next_payout_id is correct
            assert_eq!(treasury.next_payout_id, 100);
        }

        // #[ink::test]
        // fn test_add_1000_payouts() {
        //     let owner = ink::env::caller();
        //     // Set the caller to be the owner for all treasury operations
        //     ink::env::test::set_caller(owner);

        //     let mut treasury = Treasury::new();
        //     let recipient = ink::env::caller();

        //     // Add 1000 payouts with different types based on modulo
        //     for i in 0..1000u32 {
        //         let base_amount = 1_000_000u128 + (i as u128 * 1_000_000u128); // Multiples of 1e6: 1e6, 2e6, 3e6, etc.

        //         let result = match i % 3 {
        //             0 => {
        //                 // OneTime payout (i = 0, 3, 6, 9, ...)
        //                 let scheduled_block = if i % 6 == 0 { Some(100 + i) } else { None };
        //                 treasury.add_payout(recipient, U256::from(base_amount), scheduled_block)
        //             }
        //             1 => {
        //                 // Recurring payout (i = 1, 4, 7, 10, ...)
        //                 treasury.add_recurring_payout(
        //                     recipient,
        //                     U256::from(base_amount),
        //                     Some(50 + i), // start_block
        //                     20,           // interval_blocks
        //                     3,            // total_payments
        //                 )
        //             }
        //             2 => {
        //                 // Vested payout (i = 2, 5, 8, 11, ...)
        //                 treasury.add_vested_payout(
        //                     recipient,
        //                     U256::from(base_amount),
        //                     Some(200 + i), // cliff_block
        //                     60,            // vesting_duration_blocks
        //                     20,            // vesting_interval_blocks
        //                 )
        //             }
        //             _ => unreachable!(),
        //         };

        //         assert!(result.is_ok());
        //         assert_eq!(result.unwrap(), i); // Check that IDs are sequential
        //     }

        //     // Verify all payouts were added
        //     assert_eq!(treasury.get_pending_payout_ids().len(), 1000);
        //     assert_eq!(treasury.get_pending_payouts().len(), 1000);

        //     // Count each payout type
        //     let mut onetime_count = 0;
        //     let mut recurring_count = 0;
        //     let mut vested_count = 0;

        //     // Verify the payouts have correct data and types
        //     let payouts = treasury.get_pending_payouts();
        //     for (index, payout) in payouts.iter().enumerate() {
        //         let expected_amount = U256::from(1_000_000u128 + (index as u128 * 1_000_000u128));

        //         match payout {
        //             Payout::OneTime(stored) => {
        //                 onetime_count += 1;
        //                 assert_eq!(stored.id, index as u32);
        //                 assert_eq!(stored.data.to, recipient);
        //                 assert_eq!(stored.data.amount, expected_amount);

        //                 // Verify scheduling logic
        //                 if index % 6 == 0 {
        //                     assert_eq!(stored.data.scheduled_block, Some(100 + index as u32));
        //                 } else {
        //                     assert_eq!(stored.data.scheduled_block, None);
        //                 }

        //                 // Should be OneTime payout for i % 3 == 0
        //                 assert_eq!(index % 3, 0);
        //             }
        //             Payout::Recurring(stored) => {
        //                 recurring_count += 1;
        //                 assert_eq!(stored.id, index as u32);
        //                 assert_eq!(stored.data.to, recipient);
        //                 assert_eq!(stored.data.amount_per_payment, expected_amount);
        //                 assert_eq!(stored.data.start_block, Some(50 + index as u32));
        //                 assert_eq!(stored.data.interval_blocks, 20);
        //                 assert_eq!(stored.data.total_payments, 3);
        //                 assert_eq!(stored.remaining_payments, 3);

        //                 // Should be Recurring payout for i % 3 == 1
        //                 assert_eq!(index % 3, 1);
        //             }
        //             Payout::Vested(stored) => {
        //                 vested_count += 1;
        //                 assert_eq!(stored.id, index as u32);
        //                 assert_eq!(stored.data.to, recipient);
        //                 assert_eq!(stored.data.total_amount, expected_amount);
        //                 assert_eq!(stored.data.cliff_block, Some(200 + index as u32));
        //                 assert_eq!(stored.data.vesting_duration_blocks, 60);
        //                 assert_eq!(stored.data.vesting_interval_blocks, 20);
        //                 assert_eq!(stored.remaining_periods, 3); // 60/20 = 3 periods
        //                 assert_eq!(stored.original_total_periods, 3);
        //                 assert_eq!(stored.released_amount, U256::from(0));

        //                 // Should be Vested payout for i % 3 == 2
        //                 assert_eq!(index % 3, 2);
        //             }
        //         }
        //     }

        //     // Verify distribution is correct (approximately 1/3 each, accounting for 1000 % 3 = 1)
        //     assert_eq!(onetime_count, 334); // 0, 3, 6, ... (334 items: 0 to 999 with step 3)
        //     assert_eq!(recurring_count, 333); // 1, 4, 7, ... (333 items: 1 to 997 with step 3)
        //     assert_eq!(vested_count, 333); // 2, 5, 8, ... (333 items: 2 to 998 with step 3)
        //     assert_eq!(onetime_count + recurring_count + vested_count, 1000);

        //     // Verify next_payout_id is correct
        //     assert_eq!(treasury.next_payout_id, 1000);

        //     // Test type-specific getters
        //     assert_eq!(treasury.get_recurring_payouts().len(), recurring_count);
        //     assert_eq!(treasury.get_vested_payouts().len(), vested_count);
        // }

        #[ink::test]
        fn test_payout_added_event() {
            let accounts = ink::env::test::default_accounts();
            let caller = accounts.alice;

            // Set the caller to be Alice (the owner)
            ink::env::test::set_caller(caller);

            let mut treasury = Treasury::new();
            let recipient = accounts.bob;
            let amount = U256::from(5_000_000u128); // 5e6

            // Add a payout
            let result = treasury.add_payout(recipient, amount, None);
            assert!(result.is_ok());
            let payout_id = result.unwrap();

            // Check that the events were emitted (TreasuryCreated + PayoutAdded)
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);

            // Decode and verify the PayoutAdded event (index 1, after TreasuryCreated)
            let decoded_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[1].data[..],
            )
            .expect("Failed to decode PayoutAdded event");

            assert_eq!(decoded_event.payout_id, payout_id);
            assert_eq!(decoded_event.to, recipient);
            assert_eq!(decoded_event.payout_type, PayoutType::OneTime);
            assert_eq!(decoded_event.amount, amount);

            // Verify the payout data is included and correct
            match decoded_event.payout_data {
                Payout::OneTime(stored) => {
                    assert_eq!(stored.id, payout_id);
                    assert_eq!(stored.data.to, recipient);
                    assert_eq!(stored.data.amount, amount);
                    assert_eq!(stored.data.scheduled_block, None);
                    assert_eq!(stored.status, PayoutStatus::Pending);
                }
                _ => panic!("Expected OneTime payout in event data"),
            }
        }

        #[ink::test]
        fn test_multiple_payout_events() {
            let mut treasury = Treasury::new();
            let recipient1 = ink::env::caller();
            let recipient2 = H160::from([1u8; 20]);

            // Add two payouts
            treasury
                .add_payout(recipient1, U256::from(1_000_000u128), None)
                .unwrap(); // 1e6
            treasury
                .add_payout(recipient2, U256::from(2_000_000u128), None)
                .unwrap(); // 2e6

            // Check that all events were emitted (TreasuryCreated + 2 PayoutAdded)
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);

            // Verify first PayoutAdded event (index 1, after TreasuryCreated)
            let first_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[1].data[..],
            )
            .expect("Failed to decode first PayoutAdded event");
            assert_eq!(first_event.payout_id, 0);
            assert_eq!(first_event.to, recipient1);
            assert_eq!(first_event.payout_type, PayoutType::OneTime);
            assert_eq!(first_event.amount, U256::from(1_000_000u128));

            // Verify first payout data
            match first_event.payout_data {
                Payout::OneTime(stored) => {
                    assert_eq!(stored.id, 0);
                    assert_eq!(stored.data.to, recipient1);
                    assert_eq!(stored.data.amount, U256::from(1_000_000u128));
                    assert_eq!(stored.data.scheduled_block, None);
                    assert_eq!(stored.status, PayoutStatus::Pending);
                }
                _ => panic!("Expected OneTime payout in first event data"),
            }

            // Verify second PayoutAdded event (index 2)
            let second_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[2].data[..],
            )
            .expect("Failed to decode second PayoutAdded event");
            assert_eq!(second_event.payout_id, 1);
            assert_eq!(second_event.to, recipient2);
            assert_eq!(second_event.payout_type, PayoutType::OneTime);
            assert_eq!(second_event.amount, U256::from(2_000_000u128));

            // Verify second payout data
            match second_event.payout_data {
                Payout::OneTime(stored) => {
                    assert_eq!(stored.id, 1);
                    assert_eq!(stored.data.to, recipient2);
                    assert_eq!(stored.data.amount, U256::from(2_000_000u128));
                    assert_eq!(stored.data.scheduled_block, None);
                    assert_eq!(stored.status, PayoutStatus::Pending);
                }
                _ => panic!("Expected OneTime payout in second event data"),
            }
        }

        #[ink::test]
        fn test_process_payouts() {
            let accounts = ink::env::test::default_accounts();
            let caller = accounts.alice;
            let recipient1 = accounts.bob;
            let recipient2 = accounts.charlie;

            // Set the caller to be Alice (the owner)
            ink::env::test::set_caller(caller);

            let mut treasury = Treasury::new();
            let contract_address = ink::env::address();
            ink::env::test::set_account_balance(contract_address, U256::from(20_000_000)); // 20e6 - enough for all transfers

            // Add initial payouts
            let _payout_id_1 = treasury
                .add_payout(recipient1, U256::from(1_000_000), None)
                .unwrap(); // 1e6
            let _payout_id_2 = treasury
                .add_payout(recipient2, U256::from(2_000_000), None)
                .unwrap(); // 2e6
            let _payout_id_3 = treasury
                .add_payout(recipient1, U256::from(3_000_000), None)
                .unwrap(); // 3e6

            // Verify payouts are pending
            assert_eq!(treasury.get_pending_payout_ids(), vec![0, 1, 2]);
            assert_eq!(treasury.get_pending_payouts().len(), 3);

            // Process the pending payouts
            let result = treasury.process_payouts();
            assert!(result.is_ok());
            let (processed_ids, total_amount) = result.unwrap();
            assert_eq!(processed_ids, vec![0, 1, 2]);
            assert_eq!(total_amount, U256::from(6_000_000)); // 1e6 + 2e6 + 3e6

            // Verify no payouts are pending after processing
            assert_eq!(treasury.get_pending_payout_ids().len(), 0);
            assert_eq!(treasury.get_pending_payouts().len(), 0);

            // Check that all events were emitted
            // TreasuryCreated + 3 PayoutAdded + 1 PayoutsProcessed = 5 events
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 5);

            // Verify the PayoutsProcessed event (last event)
            let processed_event = <PayoutsProcessed as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[4].data[..],
            )
            .expect("Failed to decode PayoutsProcessed event");

            assert_eq!(processed_event.processed_ids, vec![0, 1, 2]);
            assert_eq!(processed_event.total_amount, U256::from(6_000_000u128)); // 1e6 + 2e6 + 3e6

            // Add new payouts after processing
            let _payout_id_4 = treasury
                .add_payout(recipient2, U256::from(4_000_000u128), None)
                .unwrap(); // 4e6
            let _payout_id_5 = treasury
                .add_payout(recipient1, U256::from(5_000_000u128), None)
                .unwrap(); // 5e6

            // Verify new payouts are pending
            assert_eq!(treasury.get_pending_payout_ids(), vec![3, 4]);
            assert_eq!(treasury.get_pending_payouts().len(), 2);

            // Process the new pending payouts
            let result = treasury.process_payouts();
            assert!(result.is_ok());
            let (second_processed_ids, second_total_amount) = result.unwrap();
            assert_eq!(second_processed_ids, vec![3, 4]);
            assert_eq!(second_total_amount, U256::from(9_000_000u128)); // 4e6 + 5e6

            // Verify no payouts are pending after second processing
            assert_eq!(treasury.get_pending_payout_ids().len(), 0);
            assert_eq!(treasury.get_pending_payouts().len(), 0);

            // Check that all events were emitted after second processing
            // TreasuryCreated + 5 PayoutAdded + 2 PayoutsProcessed = 8 events total
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 8);

            // Verify the second PayoutsProcessed event (last event)
            let second_processed_event = <PayoutsProcessed as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[7].data[..],
            )
            .expect("Failed to decode second PayoutsProcessed event");

            assert_eq!(second_processed_event.processed_ids, vec![3, 4]);
            assert_eq!(
                second_processed_event.total_amount,
                U256::from(9_000_000u128)
            ); // 4e6 + 5e6
        }

        #[ink::test]
        fn test_fund_function() {
            let treasury = setup_treasury_with_balance(2_000_000);
            let accounts = ink::env::test::default_accounts();

            // Set Charlie as the caller who will fund the treasury
            ink::env::test::set_caller(accounts.charlie);

            // Set up Charlie's account balance
            ink::env::test::set_account_balance(accounts.charlie, U256::from(5_000_000));

            // Set the value being transferred with the fund call
            let fund_amount = U256::from(1_000_000);
            ink::env::test::set_value_transferred(fund_amount);
            ink::env::test::transfer_in(fund_amount);

            let balance_after = treasury.get_balance();
            assert_eq!(balance_after, U256::from(3_000_000)); // 2M initial + 1M funded

            let charlie_balance = ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(
                accounts.charlie,
            );
            assert_eq!(charlie_balance, Ok(U256::from(4_000_000)));
        }

        #[ink::test]
        fn test_minimum_amount_validation() {
            let mut treasury = Treasury::new();
            let recipient = ink::env::caller();

            // Test amount that's too small (should fail)
            let small_amount = U256::from(100u128); // Much smaller than 1e6
            let result = treasury.add_payout(recipient, small_amount, None);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::PrecisionLoss);

            // Test amount that's not divisible by 1e6 (should fail due to precision loss)
            let non_divisible_amount = U256::from(1_000_001u128); // 1e6 + 1
            let result = treasury.add_payout(recipient, non_divisible_amount, None);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::PrecisionLoss);

            // Test minimum valid amount (should succeed)
            let min_amount = U256::from(1_000_000u128); // Exactly 1e6
            let result = treasury.add_payout(recipient, min_amount, None);
            assert!(result.is_ok());

            // Test amount larger than minimum (should succeed)
            let large_amount = U256::from(10_000_000u128); // 10e6
            let result = treasury.add_payout(recipient, large_amount, None);
            assert!(result.is_ok());
        }

        #[ink::test]
        fn test_scheduled_payouts() {
            let mut treasury = setup_treasury_with_balance(20_000_000);
            let (recipient, _) = setup_accounts();

            // Add immediate payout
            let immediate_id = treasury
                .add_payout(recipient, U256::from(1_000_000), None)
                .unwrap();

            // Add scheduled payout for future block (100)
            let future_block = 100u32;
            let scheduled_id = treasury
                .add_payout(recipient, U256::from(2_000_000), Some(future_block))
                .unwrap();

            // Verify both payouts are pending
            assert_eq!(
                treasury.get_pending_payout_ids(),
                vec![immediate_id, scheduled_id]
            );
            assert_eq!(treasury.get_pending_payouts().len(), 2);

            // Verify ready vs scheduled split
            let ready_payouts = treasury.get_ready_payouts();
            let scheduled_payouts = treasury.get_scheduled_payouts();
            assert_eq!(ready_payouts.len(), 1);
            assert_eq!(scheduled_payouts.len(), 1);
            let ready_payout_id = match &ready_payouts[0] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            let scheduled_payout_id = match &scheduled_payouts[0] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            assert_eq!(ready_payout_id, immediate_id);
            assert_eq!(scheduled_payout_id, scheduled_id);

            // Process payouts - only immediate should be processed
            let result = treasury.process_payouts();
            assert!(result.is_ok());
            let (processed_ids, total_amount) = result.unwrap();
            assert_eq!(processed_ids, vec![immediate_id]);
            assert_eq!(total_amount, U256::from(1_000_000));

            // Verify scheduled payout is still pending
            assert_eq!(treasury.get_pending_payout_ids(), vec![scheduled_id]);
            assert_eq!(treasury.get_ready_payouts().len(), 0);
            assert_eq!(treasury.get_scheduled_payouts().len(), 1);

            // Verify processed payout is in history
            assert_eq!(treasury.get_processed_payout_ids(), vec![immediate_id]);
            let retrieved_payout = treasury.get_payout(immediate_id);
            assert!(retrieved_payout.is_some());
            let payout_amount = match retrieved_payout.unwrap() {
                Payout::OneTime(stored) => stored.data.amount,
                _ => panic!("Expected OneTime payout"),
            };
            assert_eq!(payout_amount, U256::from(1_000_000));

            // Simulate block advancement by setting block number
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(future_block);

            // Now the scheduled payout should be ready
            assert_eq!(treasury.get_ready_payouts().len(), 1);
            assert_eq!(treasury.get_scheduled_payouts().len(), 0);

            // Process again - scheduled payout should now be processed
            let result = treasury.process_payouts();
            assert!(result.is_ok());
            let (processed_ids, total_amount) = result.unwrap();
            assert_eq!(processed_ids, vec![scheduled_id]);
            assert_eq!(total_amount, U256::from(2_000_000));

            // Verify all payouts are now processed
            assert_eq!(treasury.get_pending_payout_ids().len(), 0);
            assert_eq!(
                treasury.get_processed_payout_ids(),
                vec![immediate_id, scheduled_id]
            );

            // Test the dashboard utility function
            let recent_payouts = treasury.get_recent_processed_payouts(5);
            assert_eq!(recent_payouts.len(), 2); // Should get both processed payouts

            // Should be in reverse chronological order (most recent first)
            let recent_payout_0_id = match &recent_payouts[0] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            let recent_payout_1_id = match &recent_payouts[1] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            assert_eq!(recent_payout_0_id, scheduled_id); // Processed second
            assert_eq!(recent_payout_1_id, immediate_id); // Processed first

            // Test requesting more than available
            let all_recent = treasury.get_recent_processed_payouts(100);
            assert_eq!(all_recent.len(), 2);

            // Test requesting just 1
            let latest_one = treasury.get_recent_processed_payouts(1);
            assert_eq!(latest_one.len(), 1);
            let latest_payout_id = match &latest_one[0] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            assert_eq!(latest_payout_id, scheduled_id); // Most recent
        }

        #[ink::test]
        fn test_is_valid_precision_amount() {
            // Test minimum valid amount (exactly 1e6)
            assert!(Treasury::is_valid_precision_amount(U256::from(
                1_000_000u128
            )));

            // Test amounts below minimum
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                999_999u128
            )));
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                500_000u128
            )));
            assert!(!Treasury::is_valid_precision_amount(U256::from(1u128)));
            assert!(!Treasury::is_valid_precision_amount(U256::from(0u128)));

            // Test perfect divisibility (multiples of 1e6)
            assert!(Treasury::is_valid_precision_amount(U256::from(
                2_000_000u128
            ))); // 2e6
            assert!(Treasury::is_valid_precision_amount(U256::from(
                5_000_000u128
            ))); // 5e6
            assert!(Treasury::is_valid_precision_amount(U256::from(
                10_000_000u128
            ))); // 10e6
            assert!(Treasury::is_valid_precision_amount(U256::from(
                100_000_000u128
            ))); // 100e6

            // Test non-divisible amounts (should fail)
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                1_000_001u128
            ))); // 1e6 + 1
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                1_500_000u128
            ))); // 1.5e6
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                2_000_001u128
            ))); // 2e6 + 1
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                999_999_999u128
            ))); // Just under 1000e6

            // Test large valid amounts
            assert!(Treasury::is_valid_precision_amount(U256::from(
                1_000_000_000_000u128
            ))); // 1e12
            assert!(Treasury::is_valid_precision_amount(U256::from(
                1_000_000_000_000_000_000u128
            ))); // 1e18

            // Test large non-divisible amounts
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                1_000_000_000_001u128
            ))); // 1e12 + 1

            // Test boundary conditions around precision factor
            assert!(Treasury::is_valid_precision_amount(U256::from(
                999_000_000u128
            ))); // 999e6
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                999_000_001u128
            ))); // 999e6 + 1
            assert!(!Treasury::is_valid_precision_amount(U256::from(
                999_999_999u128
            ))); // Almost 1000e6

            // Test very large U256 values that are valid (divisible by 1e6)
            // Use U256 constructor for large numbers
            let large_valid = U256::from(1_000_000u128) * U256::from(1_000_000_000u128); // 1e6 * 1e9 = 1e15
            assert!(Treasury::is_valid_precision_amount(large_valid));

            // Test maximum valid precision amount we can reasonably create
            let max_reasonable = U256::from(1_000_000u128) * U256::from(u64::MAX);
            assert!(Treasury::is_valid_precision_amount(max_reasonable));

            // Test edge case: what happens with U256::MAX (should not panic)
            // This is an extreme edge case - the function should handle it gracefully
            let result = Treasury::is_valid_precision_amount(U256::MAX);
            // U256::MAX is likely not divisible by 1e6, but the function should not panic
            assert!(!result); // Almost certainly not divisible by 1e6
        }

        #[ink::test]
        fn test_get_ready_payouts() {
            let mut treasury = Treasury::new();
            let recipient = ink::env::caller();

            // Initially no ready payouts
            assert_eq!(treasury.get_ready_payouts().len(), 0);

            // Add immediate payout
            treasury
                .add_payout(recipient, U256::from(1_000_000), None)
                .unwrap();
            assert_eq!(treasury.get_ready_payouts().len(), 1);
            assert_eq!(treasury.get_scheduled_payouts().len(), 0);

            // Add scheduled payout for future block
            treasury
                .add_payout(recipient, U256::from(2_000_000), Some(1000))
                .unwrap();
            assert_eq!(treasury.get_ready_payouts().len(), 1); // Still just immediate
            assert_eq!(treasury.get_scheduled_payouts().len(), 1); // Now has scheduled

            // Simulate block advancement
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(1000);
            assert_eq!(treasury.get_ready_payouts().len(), 2); // Both ready now
            assert_eq!(treasury.get_scheduled_payouts().len(), 0); // None scheduled
        }

        #[ink::test]
        fn test_get_scheduled_payouts() {
            let mut treasury = Treasury::new();
            let recipient = ink::env::caller();

            // Initially no scheduled payouts
            assert_eq!(treasury.get_scheduled_payouts().len(), 0);

            // Add immediate payout (not scheduled)
            treasury
                .add_payout(recipient, U256::from(1_000_000), None)
                .unwrap();
            assert_eq!(treasury.get_scheduled_payouts().len(), 0);

            // Add multiple scheduled payouts
            treasury
                .add_payout(recipient, U256::from(2_000_000), Some(100))
                .unwrap();
            treasury
                .add_payout(recipient, U256::from(3_000_000), Some(200))
                .unwrap();
            treasury
                .add_payout(recipient, U256::from(4_000_000), Some(300))
                .unwrap();

            let scheduled = treasury.get_scheduled_payouts();
            assert_eq!(scheduled.len(), 3);

            // Verify they are scheduled for future blocks
            for payout in scheduled {
                match payout {
                    Payout::OneTime(stored) => {
                        let scheduled_block = stored.data.scheduled_block;
                        assert!(scheduled_block.is_some());
                        assert!(scheduled_block.unwrap() > 0);
                    }
                    _ => panic!("Expected OneTime payout for this test"),
                }
            }
        }

        #[ink::test]
        fn test_get_balance() {
            let treasury = Treasury::new();
            let contract_address = ink::env::address();

            // Set contract balance
            ink::env::test::set_account_balance(contract_address, U256::from(5_000_000));

            assert_eq!(treasury.get_balance(), U256::from(5_000_000));

            // Change balance and test again
            ink::env::test::set_account_balance(contract_address, U256::from(10_000_000));
            assert_eq!(treasury.get_balance(), U256::from(10_000_000));
        }

        #[ink::test]
        fn test_get_recent_processed_payouts_edge_cases() {
            let mut treasury = setup_treasury_with_balance(50_000_000);
            let (recipient, _) = setup_accounts();

            // Test with no processed payouts
            assert_eq!(treasury.get_recent_processed_payouts(10).len(), 0);
            assert_eq!(treasury.get_recent_processed_payouts(0).len(), 0);

            // Add and process one payout using helper
            add_and_process_payout(&mut treasury, recipient, 1_000_000);

            // Test edge cases with 1 payout
            assert_eq!(treasury.get_recent_processed_payouts(0).len(), 0);
            assert_eq!(treasury.get_recent_processed_payouts(1).len(), 1);
            assert_eq!(treasury.get_recent_processed_payouts(100).len(), 1); // More than available

            // Create more payouts using helper and test ordering
            create_multiple_processed_payouts(&mut treasury, recipient, 4); // Creates 4 more (5 total)

            let recent_3 = treasury.get_recent_processed_payouts(3);
            assert_eq!(recent_3.len(), 3);
            // Should be in reverse chronological order (most recent first)
            let recent_3_id_0 = match &recent_3[0] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            let recent_3_id_1 = match &recent_3[1] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            let recent_3_id_2 = match &recent_3[2] {
                Payout::OneTime(stored) => stored.id,
                _ => panic!("Expected OneTime payout"),
            };
            assert_eq!(recent_3_id_0, 4); // Last processed
            assert_eq!(recent_3_id_1, 3); // Second to last
            assert_eq!(recent_3_id_2, 2); // Third to last
        }

        #[ink::test]
        fn test_process_payouts_reentrancy() {
            let mut treasury = setup_treasury_with_balance(10_000_000);
            let (recipient, _) = setup_accounts();

            // Add a payout
            treasury
                .add_payout(recipient, U256::from(1_000_000), None)
                .unwrap();

            // Manually set processing flag to simulate reentrancy
            treasury.is_processing = true;

            // Should return reentrancy error
            let result = treasury.process_payouts();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::Reentrancy);

            // Reset flag and try again - should work
            treasury.is_processing = false;
            let result = treasury.process_payouts();
            assert!(result.is_ok());
        }

        #[ink::test]
        fn test_payout_status_filtering() {
            let mut treasury = setup_treasury_with_balance(10_000_000);
            let (recipient, _) = setup_accounts();

            // Add two payouts
            let id1 = treasury
                .add_payout(recipient, U256::from(1_000_000), None)
                .unwrap();
            let id2 = treasury
                .add_payout(recipient, U256::from(2_000_000), None)
                .unwrap();

            // Initially both should be pending
            assert_eq!(treasury.get_pending_payouts().len(), 2);
            assert_eq!(treasury.get_ready_payouts().len(), 2);

            // Cancel the first payout
            let result = treasury.cancel_payout(id1);
            assert!(result.is_ok());

            // Now only one should be pending
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process remaining payout
            let (processed_ids, _) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(processed_ids[0], id2);

            // No payouts should be pending now
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            assert_eq!(treasury.get_ready_payouts().len(), 0);

            // Both payouts should be in processed list (cancelled and completed)
            assert_eq!(treasury.get_processed_payout_ids().len(), 2);

            // Verify statuses
            let cancelled_payout = treasury.get_payout(id1).unwrap();
            let completed_payout = treasury.get_payout(id2).unwrap();

            match cancelled_payout {
                Payout::OneTime(stored) => {
                    assert!(matches!(stored.status, PayoutStatus::Cancelled(_)));
                }
                _ => panic!("Expected OneTime payout"),
            };
            match completed_payout {
                Payout::OneTime(stored) => {
                    assert!(matches!(stored.status, PayoutStatus::Completed(_)));
                }
                _ => panic!("Expected OneTime payout"),
            };

            // Try to cancel an already completed payout - should fail
            let result = treasury.cancel_payout(id2);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::PayoutNotFound);
        }

        #[ink::test]
        fn test_recurring_payouts() {
            let mut treasury = setup_treasury_with_balance(50_000_000);
            let alice = ink::env::test::default_accounts().alice;

            let initial_balance = U256::from(300_000_000);
            ink::env::test::set_account_balance(alice, initial_balance);

            // Add a recurring payout: 3 payments of 1e6 each, every 10 blocks, starting at block 100
            let recurring_id = treasury
                .add_recurring_payout(
                    alice,
                    U256::from(1_000_000), // amount per payment
                    Some(100),             // start at block 100
                    10,                    // every 10 blocks
                    3,                     // total 3 payments
                )
                .unwrap();

            // Should be 1 pending payout
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(treasury.get_recurring_payouts().len(), 1);
            assert_eq!(treasury.get_ready_payouts().len(), 0); // Not ready yet (block 0 < 100)

            // Advance to block 100 - first payment should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(100);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process first payment
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(processed_ids[0], recurring_id);
            assert_eq!(total_amount, U256::from(1_000_000));

            // Verify Alice received first payment (1e6)
            let balance_after_first =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(alice);
            assert_eq!(balance_after_first, Ok(U256::from(301_000_000)));

            // Should have created next recurring payment
            assert_eq!(treasury.get_pending_payouts().len(), 1); // Next payment
            assert_eq!(treasury.get_ready_payouts().len(), 0); // Not ready yet

            // Advance to block 110 - second payment should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(110);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process second payment
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(total_amount, U256::from(1_000_000));

            // Verify Alice received second payment (total: 2e6)
            let balance_after_second =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(alice);
            assert_eq!(balance_after_second, Ok(U256::from(302_000_000)));

            // Should have created third recurring payment
            assert_eq!(treasury.get_pending_payouts().len(), 1);

            // Advance to block 120 - third and final payment should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(120);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process third payment
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(total_amount, U256::from(1_000_000));

            // Verify Alice received third payment (total: 3e6)
            let final_balance =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(alice);
            assert_eq!(final_balance, Ok(U256::from(303_000_000)));

            // No more recurring payments should be created (we specified 3 total)
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            assert_eq!(treasury.get_recurring_payouts().len(), 0);

            // Should have 3 processed payouts total
            assert_eq!(treasury.get_processed_payout_ids().len(), 3);

            // Verify total amount received matches expected (3 payments × 1e6 each)
            assert_eq!(
                final_balance.unwrap(),
                initial_balance + U256::from(3_000_000)
            );
        }

        #[ink::test]
        fn test_vested_payouts() {
            let mut treasury = setup_treasury_with_balance(50_000_000);
            let bob = ink::env::test::default_accounts().bob;

            // Set Bob's initial balance to an arbitrary value
            let initial_balance = U256::from(150_000_000);
            ink::env::test::set_account_balance(bob, initial_balance);

            // Add a vested payout: 10e6 total, cliff at block 100, vesting over 30 blocks, every 10 blocks
            let vested_id = treasury
                .add_vested_payout(
                    bob,
                    U256::from(10_000_000), // total amount
                    Some(100),              // cliff at block 100
                    30,                     // vesting duration: 30 blocks
                    10,                     // vesting interval: every 10 blocks
                )
                .unwrap();

            // Should be 1 pending payout
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(treasury.get_vested_payouts().len(), 1);
            assert_eq!(treasury.get_ready_payouts().len(), 0); // Not ready yet (block 0 < 100)

            // Advance to block 100 - cliff reached, first vesting should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(100);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process first vesting payment (should be 10e6 / 3 periods = ~3.33e6 per period)
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(processed_ids[0], vested_id);
            // 10_000_000 / 3 = 3_333_333 (with integer division)
            assert_eq!(total_amount, U256::from(3_333_333));

            // Verify Bob received first vesting payment
            let balance_after_first =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(bob);
            assert_eq!(balance_after_first, Ok(U256::from(153_333_333))); // 150M + 3.333333M

            // Should have created next vesting payment
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(treasury.get_ready_payouts().len(), 0);

            // Advance to block 110 - second vesting should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(110);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process second vesting payment
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(total_amount, U256::from(3_333_333));

            // Verify Bob received second vesting payment
            let balance_after_second =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(bob);
            assert_eq!(balance_after_second, Ok(U256::from(156_666_666))); // 150M + 6.666666M (2 × 3.333333M)

            // Should have created third vesting payment
            assert_eq!(treasury.get_pending_payouts().len(), 1);

            // Advance to block 120 - third vesting should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(120);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process third vesting payment
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            // Final payment includes remainder: 10_000_000 - 6_666_666 = 3_333_334
            assert_eq!(total_amount, U256::from(3_333_334));

            // Verify Bob received third vesting payment
            let final_balance =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(bob);
            assert_eq!(final_balance, Ok(U256::from(160_000_000))); // 150M + 10M total

            // Advance to block 130 - vesting period is over (100 + 30 = 130)
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(130);

            // No more vesting payments should be created
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            assert_eq!(treasury.get_vested_payouts().len(), 0);

            // Should have 3 processed payouts total
            assert_eq!(treasury.get_processed_payout_ids().len(), 3);

            // Verify treasury balance decreased by exactly the total vested amount
            let final_treasury_balance = treasury.get_balance();
            let total_paid = U256::from(10_000_000); // Full amount: 3_333_333 + 3_333_333 + 3_333_334
            assert_eq!(final_treasury_balance, U256::from(40_000_000));

            // Verify Bob received exactly the full vested amount
            assert_eq!(final_balance.unwrap(), initial_balance + total_paid);
        }

        #[ink::test]
        fn test_mixed_payout_types() {
            let mut treasury = setup_treasury_with_balance(100_000_000);
            let (recipient, _) = setup_accounts();

            // Add different types of payouts
            let onetime_id = treasury
                .add_payout(recipient, U256::from(5_000_000), None)
                .unwrap();

            let recurring_id = treasury
                .add_recurring_payout(
                    recipient,
                    U256::from(2_000_000),
                    Some(50), // start at block 50
                    20,       // every 20 blocks
                    2,        // 2 payments total
                )
                .unwrap();

            let _vested_id = treasury
                .add_vested_payout(
                    recipient,
                    U256::from(6_000_000),
                    Some(100), // cliff at block 100
                    40,        // vesting over 40 blocks
                    20,        // every 20 blocks
                )
                .unwrap();

            // Verify counts by type
            assert_eq!(treasury.get_pending_payouts().len(), 3);

            // Count OneTime payouts manually
            let onetime_count = treasury
                .get_pending_payouts()
                .iter()
                .filter(|payout| matches!(payout, Payout::OneTime { .. }))
                .count();
            assert_eq!(onetime_count, 1);

            assert_eq!(treasury.get_recurring_payouts().len(), 1);
            assert_eq!(treasury.get_vested_payouts().len(), 1);

            // Initially only OneTime is ready
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process OneTime payout
            let (processed_ids, _) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(processed_ids[0], onetime_id);

            // Advance to block 50 - recurring should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(50);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process first recurring payment
            let (processed_ids, _) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(processed_ids[0], recurring_id);

            // Advance to block 100 - both recurring and vested should be ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(100);
            assert_eq!(treasury.get_ready_payouts().len(), 2); // Second recurring + first vested

            // Process both
            let (processed_ids, _) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 2);

            // Total processed should be 4 (1 onetime + 2 recurring + 1 vested)
            assert_eq!(treasury.get_processed_payout_ids().len(), 4);
        }

        #[ink::test]
        fn test_vested_remainder_handling() {
            // Test remainder handling: 100M in 7 tranches = 14_285_714 per period with remainder 2
            let mut treasury = setup_treasury_with_balance(150_000_000);
            let charlie = ink::env::test::default_accounts().charlie;
            ink::env::test::set_account_balance(charlie, U256::from(200_000_000));

            // Add vested payout: 100M total in 7 periods
            // 100_000_000 / 7 = 14_285_714 per period, remainder = 100_000_000 - (14_285_714 * 6) = 14_285_716
            let _vested_id = treasury
                .add_vested_payout(
                    charlie,
                    U256::from(100_000_000), // total amount: 100M
                    Some(10),                // cliff at block 10
                    70,                      // vesting duration: 70 blocks
                    10,                      // vesting interval: every 10 blocks (7 periods total)
                )
                .unwrap();

            // Process payments 1-6: should be 14_285_714 each
            for i in 1..=6 {
                ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(10 * i);
                let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
                assert_eq!(processed_ids.len(), 1);
                assert_eq!(total_amount, U256::from(14_285_714)); // 100_000_000 / 7 = 14_285_714

                let expected_balance = 200_000_000 + (14_285_714 * i as u128);
                let balance =
                    ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(charlie);
                assert_eq!(balance, Ok(U256::from(expected_balance)));
            }

            // Process payment 7: should include remainder
            // Total paid so far: 6 × 14_285_714 = 85_714_284
            // Final payment: 100_000_000 - 85_714_284 = 14_285_716
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(70);
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(total_amount, U256::from(14_285_716)); // 14_285_714 + 2 remainder

            // Verify Charlie received exactly 100M total
            let final_balance =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(charlie);
            assert_eq!(final_balance, Ok(U256::from(300_000_000))); // 200M + 100M

            // No more vesting payments should be created
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            assert_eq!(treasury.get_vested_payouts().len(), 0);

            // Verify treasury decreased by exactly 100M
            assert_eq!(treasury.get_balance(), U256::from(50_000_000)); // 150M - 100M
        }

        #[ink::test]
        fn test_add_payouts_complex_vesting() {
            // Test complex vesting: "15% Day 0 | 3 Mo Cliff | 85% Linear vesting for 27 Months"
            let mut treasury = setup_treasury_with_balance(200_000_000);
            let recipient = ink::env::test::default_accounts().alice;
            ink::env::test::set_account_balance(recipient, U256::from(100_000_000));

            let total_allocation = 100_000_000u128; // 100M tokens
            let immediate_percent = 15_000_000u128; // 15%
            let linear_percent = 85_000_000u128; // 85%
            let cliff_blocks = 90u32; // 3 months cliff (assuming ~30 blocks/month)
            let vesting_duration = 810u32; // 27 months (27 * 30)

            // Create complex vesting schedule in single call
            let payout_ids = treasury
                .add_payouts(vec![
                    // 15% immediate
                    PayoutRequest::OneTime(OneTimeData {
                        to: recipient,
                        amount: U256::from(immediate_percent),
                        scheduled_block: None, // immediate
                    }),
                    // 85% linear vesting after 3 month cliff
                    PayoutRequest::Vested(VestedData {
                        to: recipient,
                        total_amount: U256::from(linear_percent),
                        cliff_block: Some(cliff_blocks),
                        vesting_duration_blocks: vesting_duration,
                        vesting_interval_blocks: 30, // monthly releases
                    }),
                ])
                .unwrap();

            // Should have created 2 payouts
            assert_eq!(payout_ids.len(), 2);
            assert_eq!(treasury.get_pending_payouts().len(), 2);

            // Immediate payout should be ready
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process immediate 15%
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            assert_eq!(total_amount, U256::from(immediate_percent));

            // Verify recipient received 15%
            let balance =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(recipient);
            assert_eq!(balance, Ok(U256::from(100_000_000 + immediate_percent)));

            // Should have 1 pending payout left (vested)
            assert_eq!(treasury.get_pending_payouts().len(), 1);
            assert_eq!(treasury.get_ready_payouts().len(), 0); // Not ready due to cliff

            // Advance past cliff
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(cliff_blocks);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            // Process first vesting payment
            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids.len(), 1);
            // 85M / 27 months = ~3.148M per month
            let expected_monthly = linear_percent / 27;
            assert_eq!(total_amount, U256::from(expected_monthly));

            // Verify total allocation will be respected
            let total_expected = immediate_percent + linear_percent;
            assert_eq!(total_expected, total_allocation);
        }

        #[ink::test]
        fn test_add_payouts_percentage_schedule() {
            // Test: "0% Day 0 | 1 Mo Cliff | 50% | 30% | 20% each month"
            let mut treasury = setup_treasury_with_balance(200_000_000);
            let recipient = ink::env::test::default_accounts().bob;
            ink::env::test::set_account_balance(recipient, U256::from(50_000_000));

            let total_amount = 100_000_000u128;
            let month_blocks = 30u32;

            // Create percentage-based vesting schedule
            let payout_ids = treasury
                .add_payouts(vec![
                    // 50% after 1 month cliff
                    PayoutRequest::OneTime(OneTimeData {
                        to: recipient,
                        amount: U256::from(50_000_000), // 50%
                        scheduled_block: Some(month_blocks),
                    }),
                    // 30% after 2 months
                    PayoutRequest::OneTime(OneTimeData {
                        to: recipient,
                        amount: U256::from(30_000_000), // 30%
                        scheduled_block: Some(2 * month_blocks),
                    }),
                    // 20% after 3 months
                    PayoutRequest::OneTime(OneTimeData {
                        to: recipient,
                        amount: U256::from(20_000_000), // 20%
                        scheduled_block: Some(3 * month_blocks),
                    }),
                ])
                .unwrap();

            assert_eq!(payout_ids.len(), 3);
            assert_eq!(treasury.get_pending_payouts().len(), 3);
            assert_eq!(treasury.get_ready_payouts().len(), 0); // All scheduled

            // Month 1: First payment ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(month_blocks);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            let (_, amount) = treasury.process_payouts().unwrap();
            assert_eq!(amount, U256::from(50_000_000));

            // Month 2: Second payment ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(2 * month_blocks);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            let (_, amount) = treasury.process_payouts().unwrap();
            assert_eq!(amount, U256::from(30_000_000));

            // Month 3: Final payment ready
            ink::env::test::set_block_number::<ink::env::DefaultEnvironment>(3 * month_blocks);
            assert_eq!(treasury.get_ready_payouts().len(), 1);

            let (_, amount) = treasury.process_payouts().unwrap();
            assert_eq!(amount, U256::from(20_000_000));

            // All payouts completed
            assert_eq!(treasury.get_pending_payouts().len(), 0);
            assert_eq!(treasury.get_processed_payout_ids().len(), 3);

            // Verify recipient received full amount
            let final_balance =
                ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(recipient);
            assert_eq!(final_balance, Ok(U256::from(50_000_000 + total_amount)));
        }

        #[ink::test]
        fn test_add_payouts_validation() {
            let mut treasury = setup_treasury_with_balance(100_000_000);
            let recipient = ink::env::test::default_accounts().charlie;

            // Test validation failure - should reject all payouts if any is invalid
            let result = treasury.add_payouts(vec![
                PayoutRequest::OneTime(OneTimeData {
                    to: recipient,
                    amount: U256::from(10_000_000), // Valid
                    scheduled_block: None,
                }),
                PayoutRequest::OneTime(OneTimeData {
                    to: recipient,
                    amount: U256::from(100), // Invalid - too small
                    scheduled_block: None,
                }),
            ]);

            assert_eq!(result, Err(Error::PrecisionLoss));
            assert_eq!(treasury.get_pending_payouts().len(), 0); // No payouts created
        }

        #[ink::test]
        fn test_comprehensive_payout_events() {
            let mut treasury = setup_treasury_with_balance(100_000_000);
            let recipient = ink::env::test::default_accounts().alice;

            // Add scheduled OneTime payout
            treasury
                .add_payout(recipient, U256::from(10_000_000), Some(100))
                .unwrap();

            // Add Recurring payout
            treasury
                .add_recurring_payout(recipient, U256::from(5_000_000), Some(50), 20, 3)
                .unwrap();

            // Add Vested payout
            treasury
                .add_vested_payout(recipient, U256::from(15_000_000), Some(200), 60, 20)
                .unwrap();

            // Should have 4 events: TreasuryCreated + 3 PayoutAdded
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 4);

            // Verify OneTime event (index 1)
            let onetime_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[1].data[..],
            )
            .expect("Failed to decode OneTime PayoutAdded event");
            assert_eq!(onetime_event.payout_type, PayoutType::OneTime);
            assert_eq!(onetime_event.amount, U256::from(10_000_000));
            assert_eq!(onetime_event.to, recipient);

            // Verify OneTime payout data
            match onetime_event.payout_data {
                Payout::OneTime(stored) => {
                    assert_eq!(stored.data.scheduled_block, Some(100));
                    assert_eq!(stored.status, PayoutStatus::Pending);
                }
                _ => panic!("Expected OneTime payout in event data"),
            }

            // Verify Recurring event (index 2)
            let recurring_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[2].data[..],
            )
            .expect("Failed to decode Recurring PayoutAdded event");
            assert_eq!(recurring_event.payout_type, PayoutType::Recurring);
            assert_eq!(recurring_event.amount, U256::from(5_000_000));
            assert_eq!(recurring_event.to, recipient);

            // Verify Recurring payout data
            match recurring_event.payout_data {
                Payout::Recurring(stored) => {
                    assert_eq!(stored.data.start_block, Some(50));
                    assert_eq!(stored.data.interval_blocks, 20);
                    assert_eq!(stored.data.total_payments, 3);
                    assert_eq!(stored.remaining_payments, 3);
                    assert_eq!(stored.status, PayoutStatus::Pending);
                }
                _ => panic!("Expected Recurring payout in event data"),
            }

            // Verify Vested event (index 3)
            let vested_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[3].data[..],
            )
            .expect("Failed to decode Vested PayoutAdded event");
            assert_eq!(vested_event.payout_type, PayoutType::Vested);
            assert_eq!(vested_event.amount, U256::from(15_000_000));
            assert_eq!(vested_event.to, recipient);

            // Verify Vested payout data
            match vested_event.payout_data {
                Payout::Vested(stored) => {
                    assert_eq!(stored.data.cliff_block, Some(200));
                    assert_eq!(stored.data.vesting_duration_blocks, 60);
                    assert_eq!(stored.data.vesting_interval_blocks, 20);
                    assert_eq!(stored.remaining_periods, 3); // 60/20 = 3
                    assert_eq!(stored.original_total_periods, 3);
                    assert_eq!(stored.released_amount, U256::from(0));
                    assert_eq!(stored.status, PayoutStatus::Pending);
                }
                _ => panic!("Expected Vested payout in event data"),
            }
        }
    }
}

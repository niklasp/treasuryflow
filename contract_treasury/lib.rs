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
    pub struct Payout {
        id: u32,
        to: H160,
        amount: U256,
        scheduled_block: Option<u32>, // None = immediate, Some = scheduled for future block
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
        amount: U256,
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
        /// Amount too small (precision loss risk)
        AmountTooSmall = 8,
    }

    impl Treasury {
        #[ink(constructor)]
        pub fn new(owner: H160) -> Self {
            let instance = Self {
                owner,
                pending_payout_ids: Vec::new(),
                payouts: StorageVec::new(),
                processed_payout_ids: Vec::new(),
                archived_payouts: Mapping::new(),
                is_processing: false,
                next_payout_id: 0,
            };

            Self::env().emit_event(TreasuryCreated { owner });

            instance
        }

        /// Helper function to check if a payout is ready to be processed
        fn is_ready(&self, payout: &Payout) -> bool {
            match payout.scheduled_block {
                None => true, // Immediate payout
                Some(block) => self.env().block_number() >= block,
            }
        }

        /// Move a processed payout to history
        fn move_to_processed(&mut self, payout: Payout) {
            // Store in archived payouts (always accessible by ID)
            self.archived_payouts.insert(payout.id, &payout);

            // Add to complete processed IDs list (no limit)
            self.processed_payout_ids.push(payout.id);
        }

        /// Safely validate amount for precision requirements without arithmetic side effects
        fn is_valid_precision_amount(amount: U256) -> bool {
            const MIN_AMOUNT: u128 = 1_000_000; // 1e6 - minimum to avoid precision loss
            const PRECISION_FACTOR: U256 = U256([1_000_000, 0, 0, 0]); // 1e6 as U256

            // Check minimum amount first
            if amount < U256::from(MIN_AMOUNT) {
                return false;
            }

            // Safe divisibility check: divide and multiply back, compare with original
            // This avoids modulo operation entirely
            let divided = amount / PRECISION_FACTOR;
            let multiplied_back = divided * PRECISION_FACTOR;

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
                            if payout.id == id {
                                return Some(payout);
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
            let start_index = if count >= total_processed {
                0
            } else {
                total_processed - count
            };

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
                    if payout.id == id {
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

        #[ink(message)]
        pub fn add_payout(&mut self, to: H160, amount: U256) -> Result<u32, Error> {
            // Validate amount for precision safety
            if !Self::is_valid_precision_amount(amount) {
                return Err(Error::AmountTooSmall);
            }

            let id = self.next_payout_id;
            let payout = Payout {
                id,
                to,
                amount,
                scheduled_block: None,
            };

            self.payouts.push(&payout);
            self.pending_payout_ids.push(id);

            self.env().emit_event(PayoutAdded {
                payout_id: id,
                to,
                amount,
            });

            self.next_payout_id = self.next_payout_id.saturating_add(1);
            Ok(id)
        }

        #[ink(message)]
        pub fn add_scheduled_payout(
            &mut self,
            to: H160,
            amount: U256,
            block_number: u32,
        ) -> Result<u32, Error> {
            // Validate amount for precision safety
            if !Self::is_valid_precision_amount(amount) {
                return Err(Error::AmountTooSmall);
            }

            let id = self.next_payout_id;
            let payout = Payout {
                id,
                to,
                amount,
                scheduled_block: Some(block_number),
            };

            self.payouts.push(&payout);
            self.pending_payout_ids.push(id);

            self.env().emit_event(PayoutAdded {
                payout_id: id,
                to,
                amount,
            });

            self.next_payout_id = self.next_payout_id.saturating_add(1);
            Ok(id)
        }

        #[ink(message)]
        pub fn add_payout_batch(&mut self, payouts: Vec<(H160, U256)>) -> Result<Vec<u32>, Error> {
            let mut ids = Vec::new();
            for (to, amount) in payouts {
                let id = self.add_payout(to, amount)?;
                ids.push(id);
            }
            Ok(ids)
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

            // Find ready payouts (only those that are ready to be processed)
            let pending_ids = self.pending_payout_ids.clone();
            for payout_id in pending_ids.iter() {
                // Find the payout by ID
                for i in 0..self.payouts.len() {
                    if let Some(payout) = self.payouts.get(i) {
                        if payout.id == *payout_id && self.is_ready(&payout) {
                            ready_payouts.push(payout.clone());
                            total_amount = total_amount.saturating_add(payout.amount);
                            break;
                        }
                    }
                }
            }

            // Process only the ready payouts
            let mut processed_ids = Vec::new();
            for payout in ready_payouts.iter() {
                let transfer_result = self.env().transfer(payout.to, payout.amount);
                if transfer_result.is_err() {
                    self.is_processing = false;
                    return Err(Error::InsufficientBalance);
                }
                processed_ids.push(payout.id);
            }

            // Move processed payouts to history
            for payout in ready_payouts {
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
            let treasury = Treasury::new(ink::env::caller());
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
            let id = treasury.add_payout(to, U256::from(amount)).unwrap();
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

        fn add_scheduled_payout(
            treasury: &mut Treasury,
            to: H160,
            amount: u128,
            block_number: u32,
        ) -> u32 {
            treasury
                .add_scheduled_payout(to, U256::from(amount), block_number)
                .unwrap()
        }

        #[ink::test]
        fn default_works() {
            let treasury = Treasury::new(ink::env::caller());
            assert!(!treasury.get_processing());
        }

        #[ink::test]
        fn it_works() {
            let mut treasury = Treasury::new(ink::env::caller());
            assert!(!treasury.get_processing());

            treasury
                .add_payout(ink::env::caller(), U256::from(1_000_000u128)) // 1e6 - minimum amount
                .unwrap();
            assert!(treasury.get_pending_payouts().len() == 1);

            let (processed_ids, total_amount) = treasury.process_payouts().unwrap();
            assert_eq!(processed_ids, vec![0]);
            assert_eq!(total_amount, U256::from(1_000_000u128));
            assert!(treasury.get_pending_payouts().len() == 0);
        }

        #[ink::test]
        fn test_add_100_payouts() {
            let mut treasury = Treasury::new(ink::env::caller());
            let recipient = ink::env::caller();

            // Add 100 payouts
            for i in 0..100u32 {
                let amount = 1_000_000u128 + (i as u128 * 1_000_000u128); // Multiples of 1e6: 1e6, 2e6, 3e6, etc.
                let result = treasury.add_payout(recipient, U256::from(amount));
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), i); // Check that IDs are sequential
            }

            // Verify all payouts were added
            assert_eq!(treasury.get_pending_payout_ids().len(), 100);
            assert_eq!(treasury.get_pending_payouts().len(), 100);

            // Verify the payouts have correct data
            let payouts = treasury.get_pending_payouts();
            for (index, payout) in payouts.iter().enumerate() {
                assert_eq!(payout.id, index as u32);
                assert_eq!(payout.to, recipient);
                assert_eq!(
                    payout.amount,
                    U256::from(1_000_000u128 + (index as u128 * 1_000_000u128))
                );
            }

            // Verify next_payout_id is correct
            assert_eq!(treasury.next_payout_id, 100);
        }

        // #[ink::test]
        // fn test_add_1000_payouts() {
        //     let mut treasury = Treasury::new(ink::env::caller());
        //     let recipient = ink::env::caller();

        //     // Add 1000 payouts
        //     for i in 0..1000u32 {
        //         let amount = 1_000_000u128 + (i as u128 * 1_000_000u128); // Multiples of 1e6: 1e6, 2e6, 3e6, etc.
        //         let result = treasury.add_payout(recipient, U256::from(amount));
        //         assert!(result.is_ok());
        //         assert_eq!(result.unwrap(), i); // Check that IDs are sequential
        //     }

        //     // Verify all payouts were added
        //     assert_eq!(treasury.get_pending_payout_ids().len(), 1000);
        //     assert_eq!(treasury.get_pending_payouts().len(), 1000);

        //     // Verify the payouts have correct data
        //     let payouts = treasury.get_pending_payouts();
        //     for (index, payout) in payouts.iter().enumerate() {
        //         assert_eq!(payout.id, index as u32);
        //         assert_eq!(payout.to, recipient);
        //         assert_eq!(
        //             payout.amount,
        //             U256::from(1_000_000u128 + (index as u128 * 1_000_000u128))
        //         );
        //     }

        //     // Verify next_payout_id is correct
        //     assert_eq!(treasury.next_payout_id, 1000);
        // }

        #[ink::test]
        fn test_payout_added_event() {
            let accounts = ink::env::test::default_accounts();
            let caller = accounts.alice;

            let mut treasury = Treasury::new(caller);
            let recipient = accounts.bob;
            let amount = U256::from(5_000_000u128); // 5e6

            // Add a payout
            let result = treasury.add_payout(recipient, amount);
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
            assert_eq!(decoded_event.amount, amount);
        }

        #[ink::test]
        fn test_multiple_payout_events() {
            let mut treasury = Treasury::new(ink::env::caller());
            let recipient1 = ink::env::caller();
            let recipient2 = H160::from([1u8; 20]);

            // Add two payouts
            treasury
                .add_payout(recipient1, U256::from(1_000_000u128))
                .unwrap(); // 1e6
            treasury
                .add_payout(recipient2, U256::from(2_000_000u128))
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
            assert_eq!(first_event.amount, U256::from(1_000_000u128));

            // Verify second PayoutAdded event (index 2)
            let second_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[2].data[..],
            )
            .expect("Failed to decode second PayoutAdded event");
            assert_eq!(second_event.payout_id, 1);
            assert_eq!(second_event.to, recipient2);
            assert_eq!(second_event.amount, U256::from(2_000_000u128));
        }

        #[ink::test]
        fn test_process_payouts() {
            let accounts = ink::env::test::default_accounts();
            let caller = accounts.alice;
            let recipient1 = accounts.bob;
            let recipient2 = accounts.charlie;

            let mut treasury = Treasury::new(caller);
            let contract_address = ink::env::address();
            ink::env::test::set_account_balance(contract_address, U256::from(20_000_000)); // 20e6 - enough for all transfers

            // Add initial payouts
            let _payout_id_1 = treasury
                .add_payout(recipient1, U256::from(1_000_000))
                .unwrap(); // 1e6
            let _payout_id_2 = treasury
                .add_payout(recipient2, U256::from(2_000_000))
                .unwrap(); // 2e6
            let _payout_id_3 = treasury
                .add_payout(recipient1, U256::from(3_000_000))
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
                .add_payout(recipient2, U256::from(4_000_000u128))
                .unwrap(); // 4e6
            let _payout_id_5 = treasury
                .add_payout(recipient1, U256::from(5_000_000u128))
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
            let accounts = ink::env::test::default_accounts();
            let caller = accounts.alice;

            let mut treasury = Treasury::new(caller);

            // Set transferred value for testing
            ink::env::test::set_value_transferred(U256::from(1000));

            let result = treasury.fund();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), U256::from(1000));

            // Check that the FundsAdded event was emitted
            // TreasuryCreated + FundsAdded = 2 events
            let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);

            // Verify the FundsAdded event (index 1, after TreasuryCreated)
            let funds_event = <FundsAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[1].data[..],
            )
            .expect("Failed to decode FundsAdded event");

            assert_eq!(funds_event.amount, U256::from(1000));
            // Note: In test environment, the caller conversion might not match exactly
        }

        #[ink::test]
        fn test_minimum_amount_validation() {
            let mut treasury = Treasury::new(ink::env::caller());
            let recipient = ink::env::caller();

            // Test amount that's too small (should fail)
            let small_amount = U256::from(100u128); // Much smaller than 1e6
            let result = treasury.add_payout(recipient, small_amount);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::AmountTooSmall);

            // Test amount that's not divisible by 1e6 (should fail due to precision loss)
            let non_divisible_amount = U256::from(1_000_001u128); // 1e6 + 1
            let result = treasury.add_payout(recipient, non_divisible_amount);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::AmountTooSmall);

            // Test minimum valid amount (should succeed)
            let min_amount = U256::from(1_000_000u128); // Exactly 1e6
            let result = treasury.add_payout(recipient, min_amount);
            assert!(result.is_ok());

            // Test amount larger than minimum (should succeed)
            let large_amount = U256::from(10_000_000u128); // 10e6
            let result = treasury.add_payout(recipient, large_amount);
            assert!(result.is_ok());
        }

        #[ink::test]
        fn test_scheduled_payouts() {
            let mut treasury = setup_treasury_with_balance(20_000_000);
            let (recipient, _) = setup_accounts();

            // Add immediate payout
            let immediate_id = treasury
                .add_payout(recipient, U256::from(1_000_000))
                .unwrap();

            // Add scheduled payout for future block (100)
            let future_block = 100u32;
            let scheduled_id =
                add_scheduled_payout(&mut treasury, recipient, 2_000_000, future_block);

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
            assert_eq!(ready_payouts[0].id, immediate_id);
            assert_eq!(scheduled_payouts[0].id, scheduled_id);

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
            assert_eq!(retrieved_payout.unwrap().amount, U256::from(1_000_000));

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
            assert_eq!(recent_payouts[0].id, scheduled_id); // Processed second
            assert_eq!(recent_payouts[1].id, immediate_id); // Processed first

            // Test requesting more than available
            let all_recent = treasury.get_recent_processed_payouts(100);
            assert_eq!(all_recent.len(), 2);

            // Test requesting just 1
            let latest_one = treasury.get_recent_processed_payouts(1);
            assert_eq!(latest_one.len(), 1);
            assert_eq!(latest_one[0].id, scheduled_id); // Most recent
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
            let mut treasury = Treasury::new(ink::env::caller());
            let recipient = ink::env::caller();

            // Initially no ready payouts
            assert_eq!(treasury.get_ready_payouts().len(), 0);

            // Add immediate payout
            treasury
                .add_payout(recipient, U256::from(1_000_000))
                .unwrap();
            assert_eq!(treasury.get_ready_payouts().len(), 1);
            assert_eq!(treasury.get_scheduled_payouts().len(), 0);

            // Add scheduled payout for future block
            treasury
                .add_scheduled_payout(recipient, U256::from(2_000_000), 1000)
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
            let mut treasury = Treasury::new(ink::env::caller());
            let recipient = ink::env::caller();

            // Initially no scheduled payouts
            assert_eq!(treasury.get_scheduled_payouts().len(), 0);

            // Add immediate payout (not scheduled)
            treasury
                .add_payout(recipient, U256::from(1_000_000))
                .unwrap();
            assert_eq!(treasury.get_scheduled_payouts().len(), 0);

            // Add multiple scheduled payouts
            treasury
                .add_scheduled_payout(recipient, U256::from(2_000_000), 100)
                .unwrap();
            treasury
                .add_scheduled_payout(recipient, U256::from(3_000_000), 200)
                .unwrap();
            treasury
                .add_scheduled_payout(recipient, U256::from(4_000_000), 300)
                .unwrap();

            let scheduled = treasury.get_scheduled_payouts();
            assert_eq!(scheduled.len(), 3);

            // Verify they are scheduled for future blocks
            for payout in scheduled {
                assert!(payout.scheduled_block.is_some());
                assert!(payout.scheduled_block.unwrap() > 0);
            }
        }

        #[ink::test]
        fn test_get_balance() {
            let treasury = Treasury::new(ink::env::caller());
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
            assert_eq!(recent_3[0].id, 4); // Last processed
            assert_eq!(recent_3[1].id, 3); // Second to last
            assert_eq!(recent_3[2].id, 2); // Third to last
        }

        #[ink::test]
        fn test_add_payout_batch() {
            let mut treasury = Treasury::new(ink::env::caller());
            let (recipient1, recipient2) = setup_accounts();

            // Test empty batch
            let result = treasury.add_payout_batch(vec![]);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().len(), 0);

            // Test single payout batch
            let single_batch = vec![(recipient1, U256::from(1_000_000))];
            let result = treasury.add_payout_batch(single_batch);
            assert!(result.is_ok());
            let ids = result.unwrap();
            assert_eq!(ids.len(), 1);
            assert_eq!(ids[0], 0);

            // Test multiple payouts batch
            let multi_batch = vec![
                (recipient1, U256::from(2_000_000)),
                (recipient2, U256::from(3_000_000)),
                (recipient1, U256::from(4_000_000)),
            ];
            let result = treasury.add_payout_batch(multi_batch);
            assert!(result.is_ok());
            let ids = result.unwrap();
            assert_eq!(ids.len(), 3);
            assert_eq!(ids, vec![1, 2, 3]); // Sequential IDs

            // Verify all payouts were added
            assert_eq!(treasury.get_pending_payouts().len(), 4); // 1 + 3 = 4 total

            // Test batch with invalid amount (should fail on first invalid, but keep previous valid ones)
            let initial_count = treasury.get_pending_payouts().len(); // Should be 4
            let invalid_batch = vec![
                (recipient1, U256::from(5_000_000)), // Valid
                (recipient2, U256::from(100)),       // Invalid (too small)
            ];
            let result = treasury.add_payout_batch(invalid_batch);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::AmountTooSmall);

            // Should have initial_count + 1 (first valid payout was added before failure)
            assert_eq!(treasury.get_pending_payouts().len(), initial_count + 1);
        }

        #[ink::test]
        fn test_process_payouts_reentrancy() {
            let mut treasury = setup_treasury_with_balance(10_000_000);
            let (recipient, _) = setup_accounts();

            // Add a payout
            treasury
                .add_payout(recipient, U256::from(1_000_000))
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
    }
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod treasury {
    use ink::prelude::vec::Vec;
    use ink::storage::StorageVec;
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
    }

    #[ink(storage)]
    pub struct Treasury {
        owner: H160,
        pending_payout_ids: Vec<u32>,
        payouts: StorageVec<Payout>,
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
                is_processing: false,
                next_payout_id: 0,
            };

            Self::env().emit_event(TreasuryCreated { owner });

            instance
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
            // Check for minimum amount to avoid precision loss during transfers
            // The precision loss occurs when converting from 18 decimals to 12 decimals (6 digit difference)
            // So amounts must be divisible by 1e6 to avoid losing precision
            const MIN_AMOUNT: u128 = 1_000_000; // 1e6 - minimum to avoid precision loss
            if amount < U256::from(MIN_AMOUNT)
                || amount % U256::from(1_000_000u128) != U256::from(0)
            {
                return Err(Error::AmountTooSmall);
            }

            let id = self.next_payout_id;
            let payout = Payout { id, to, amount };

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
        pub fn process_pending_payouts(&mut self) -> Result<(Vec<u32>, U256), Error> {
            // Reentrancy guard
            if self.is_processing {
                return Err(Error::Reentrancy);
            }
            self.is_processing = true;

            // Process pending payouts
            let pending_ids = self.pending_payout_ids.clone();
            let mut total_amount: U256 = U256::from(0);

            for payout_id in pending_ids.iter() {
                // Find the payout by ID
                let mut payout = None;
                for i in 0..self.payouts.len() {
                    if let Some(p) = self.payouts.get(i) {
                        if p.id == *payout_id {
                            payout = Some(p);
                            break;
                        }
                    }
                }

                if let Some(payout) = payout {
                    let transfer_result = self.env().transfer(payout.to, payout.amount);
                    if transfer_result.is_err() {
                        self.is_processing = false;
                        return Err(Error::InsufficientBalance);
                    }
                    // Add to total amount during iteration
                    total_amount = total_amount.saturating_add(payout.amount);
                }
            }

            // Emit event with processed IDs and total amount before clearing
            self.env().emit_event(PayoutsProcessed {
                processed_ids: pending_ids.clone(),
                total_amount,
            });

            // Clear pending payouts after successful processing
            self.pending_payout_ids.clear();
            self.is_processing = false;

            Ok((pending_ids, total_amount))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

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

            let (processed_ids, total_amount) = treasury.process_pending_payouts().unwrap();
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
        fn test_process_pending_payouts() {
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
            let result = treasury.process_pending_payouts();
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
            let result = treasury.process_pending_payouts();
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
    }
}

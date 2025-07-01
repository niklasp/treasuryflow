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
        amount: Balance,
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
        amount: Balance,
    }

    #[ink(event)]
    pub struct PayoutsProcessed {
        processed_ids: Vec<u32>,
        total_amount: Balance,
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
        pub fn get_balance(&self) -> u128 {
            self.env().balance().as_u128()
        }

        #[ink(message)]
        pub fn add_payout(&mut self, to: H160, amount: Balance) -> Result<u32, Error> {
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
        pub fn add_payout_batch(
            &mut self,
            payouts: Vec<(H160, Balance)>,
        ) -> Result<Vec<u32>, Error> {
            let mut ids = Vec::new();
            for (to, amount) in payouts {
                let id = self.add_payout(to, amount)?;
                ids.push(id);
            }
            Ok(ids)
        }

        #[ink(message)]
        pub fn process_pending_payouts(&mut self) -> Result<(Vec<u32>, Balance), Error> {
            // Reentrancy guard
            if self.is_processing {
                return Err(Error::Reentrancy);
            }
            self.is_processing = true;

            // Process pending payouts
            let pending_ids = self.pending_payout_ids.clone();
            let mut total_amount: Balance = 0;

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
                    let transfer_result = self.env().transfer(payout.to, U256::from(payout.amount));
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

            treasury.add_payout(ink::env::caller(), 100).unwrap();
            assert!(treasury.get_pending_payouts().len() == 1);

            let (processed_ids, total_amount) = treasury.process_pending_payouts().unwrap();
            assert_eq!(processed_ids, vec![0]);
            assert_eq!(total_amount, 100);
            assert!(treasury.get_pending_payouts().len() == 0);
        }

        #[ink::test]
        fn test_add_100_payouts() {
            let mut treasury = Treasury::new(ink::env::caller());
            let recipient = ink::env::caller();

            // Add 100 payouts
            for i in 0..100u32 {
                let amount = (i + 1) * 10; // Different amounts: 10, 20, 30, ..., 1000
                let result = treasury.add_payout(recipient, amount as u128);
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
                assert_eq!(payout.amount, ((index + 1) * 10) as u128);
            }

            // Verify next_payout_id is correct
            assert_eq!(treasury.next_payout_id, 100);
        }

        #[ink::test]
        fn test_add_1000_payouts() {
            let mut treasury = Treasury::new(ink::env::caller());
            let recipient = ink::env::caller();

            // Add 100 payouts
            for i in 0..1000u32 {
                let amount = (i + 1) * 10; // Different amounts: 10, 20, 30, ..., 1000
                let result = treasury.add_payout(recipient, amount as u128);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), i); // Check that IDs are sequential
            }

            // Verify all payouts were added
            assert_eq!(treasury.get_pending_payout_ids().len(), 1000);
            assert_eq!(treasury.get_pending_payouts().len(), 1000);

            // Verify the payouts have correct data
            let payouts = treasury.get_pending_payouts();
            for (index, payout) in payouts.iter().enumerate() {
                assert_eq!(payout.id, index as u32);
                assert_eq!(payout.to, recipient);
                assert_eq!(payout.amount, ((index + 1) * 10) as u128);
            }

            // Verify next_payout_id is correct
            assert_eq!(treasury.next_payout_id, 1000);
        }

        #[ink::test]
        fn test_payout_added_event() {
            let accounts = ink::env::test::default_accounts();
            let caller = accounts.alice;

            let mut treasury = Treasury::new(caller);
            let recipient = accounts.bob;
            let amount = 500u128;

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
            treasury.add_payout(recipient1, 100u128).unwrap();
            treasury.add_payout(recipient2, 200u128).unwrap();

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
            assert_eq!(first_event.amount, 100u128);

            // Verify second PayoutAdded event (index 2)
            let second_event = <PayoutAdded as parity_scale_codec::Decode>::decode(
                &mut &emitted_events[2].data[..],
            )
            .expect("Failed to decode second PayoutAdded event");
            assert_eq!(second_event.payout_id, 1);
            assert_eq!(second_event.to, recipient2);
            assert_eq!(second_event.amount, 200u128);
        }

        #[ink::test]
        fn test_process_pending_payouts() {
            let accounts = ink::env::test::default_accounts();
            let caller = accounts.alice;
            let recipient1 = accounts.bob;
            let recipient2 = accounts.charlie;

            let mut treasury = Treasury::new(caller);

            // Add initial payouts
            let _payout_id_1 = treasury.add_payout(recipient1, 100u128).unwrap();
            let _payout_id_2 = treasury.add_payout(recipient2, 200u128).unwrap();
            let _payout_id_3 = treasury.add_payout(recipient1, 300u128).unwrap();

            // Verify payouts are pending
            assert_eq!(treasury.get_pending_payout_ids(), vec![0, 1, 2]);
            assert_eq!(treasury.get_pending_payouts().len(), 3);

            // Process the pending payouts
            let result = treasury.process_pending_payouts();
            assert!(result.is_ok());
            let (processed_ids, total_amount) = result.unwrap();
            assert_eq!(processed_ids, vec![0, 1, 2]);
            assert_eq!(total_amount, 600); // 100 + 200 + 300

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
            assert_eq!(processed_event.total_amount, 600); // 100 + 200 + 300

            // Add new payouts after processing
            let _payout_id_4 = treasury.add_payout(recipient2, 400u128).unwrap();
            let _payout_id_5 = treasury.add_payout(recipient1, 500u128).unwrap();

            // Verify new payouts are pending
            assert_eq!(treasury.get_pending_payout_ids(), vec![3, 4]);
            assert_eq!(treasury.get_pending_payouts().len(), 2);

            // Process the new pending payouts
            let result = treasury.process_pending_payouts();
            assert!(result.is_ok());
            let (second_processed_ids, second_total_amount) = result.unwrap();
            assert_eq!(second_processed_ids, vec![3, 4]);
            assert_eq!(second_total_amount, 900); // 400 + 500

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
            assert_eq!(second_processed_event.total_amount, 900); // 400 + 500
        }
    }
}

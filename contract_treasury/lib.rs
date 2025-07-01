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
        processing: bool,
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
        payout_id: u32,
        to: H160,
        amount: Balance,
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
                processing: false,
                next_payout_id: 0,
            };

            Self::env().emit_event(TreasuryCreated { owner });

            instance
        }

        #[ink(message)]
        pub fn get_processing(&self) -> bool {
            self.processing
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
        pub fn process_pending_payouts(&mut self) -> Result<(), Error> {
            // Reentrancy guard
            if self.processing {
                return Err(Error::Reentrancy);
            }
            self.processing = true;

            // Process pending payouts
            let pending_ids = self.pending_payout_ids.clone();
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
                        self.processing = false;
                        return Err(Error::InsufficientBalance);
                    }
                }
            }

            // Clear pending payouts after successful processing
            self.pending_payout_ids.clear();
            self.processing = false;

            Ok(())
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

            treasury.process_pending_payouts().unwrap();
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
    }
}

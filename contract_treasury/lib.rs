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
            Self {
                owner,
                pending_payout_ids: Vec::new(),
                payouts: StorageVec::new(),
                processing: false,
                next_payout_id: 0,
            }
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
        pub fn add_payout(&mut self, to: H160, amount: Balance) -> Result<u32, Error> {
            let id = self.next_payout_id;
            let payout = Payout { id, to, amount };

            self.payouts.push(&payout);
            self.pending_payout_ids.push(id);

            self.next_payout_id = self.next_payout_id.saturating_add(1);
            Ok(id)
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
    }
}

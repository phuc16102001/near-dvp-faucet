use near_sdk::BorshStorageKey;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AccountKey
}
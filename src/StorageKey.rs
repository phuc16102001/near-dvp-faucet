use near_sdk::{BorshDeserialize, BorshSerialize, BorshStorageKey};

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    AccountKey
}
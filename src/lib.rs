use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{
    collections::LookupMap,
    env, ext_contract, near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, Balance, Gas, PanicOnDefault, Promise, PromiseOrValue,
};

use types::StorageKey;
mod types;

const ONE_YOCTO_NEAR: Balance = 1;
pub const FT_TRANSFER_GAS: Gas = 10_000_000_000_000;
pub const FAUCET_CALLBACK_GAS: Gas = 10_000_000_000_000;

// ====================================== Faucet info =================================

// To cast to JSON
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FaucetInfo {
    pub current_shared_balance: U128,
    pub available_balance: U128,
    pub total_share_account: U128,
    pub max_share_per_account: U128,
    pub is_paused: bool,
}

impl FaucetInfo {
    pub fn from(contract: &FaucetContract) -> Self {
        FaucetInfo {
            current_shared_balance: U128(contract.current_shared_balance),
            available_balance: U128(contract.available_balance),
            total_share_account: U128(contract.total_share_account),
            max_share_per_account: U128(contract.max_share_per_account),
            is_paused: contract.is_paused,
        }
    }
}

// ====================================== External interface ====================================

// Token transfering method of NEP141
#[ext_contract(ext_ft_contract)]
pub trait FungibleTokenCore {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

// For callback after transfer token
#[ext_contract(ext_self)]
pub trait ExtFaucetContract {
    fn ft_transfer_callback(&mut self, amount: U128, account_id: AccountId);
}

// For transfering balance share from owner
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

// ======================================== Faucet contract =====================================

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FaucetContract {
    pub owner_id: AccountId,             // Contract owner (for transfer tokens)
    pub ft_contract_id: AccountId,       // FT contract account
    pub current_shared_balance: Balance, // Shared tokens
    pub available_balance: Balance,      // Current available to faucet
    pub total_share_account: Balance,    // Number of sharing accounts
    pub accounts: LookupMap<AccountId, Balance>, // Balance of account which was achieved (no storage deposit)
    pub max_share_per_account: Balance,          // Max balance fauceting for each account
    pub is_paused: bool,                         // Contract status
}

#[near_bindgen]
impl FaucetContract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        ft_contract_id: AccountId,
        max_share_per_account: Balance,
    ) -> Self {
        FaucetContract {
            owner_id,
            ft_contract_id,
            current_shared_balance: 0,
            available_balance: 0,
            total_share_account: 0,
            accounts: LookupMap::new(StorageKey::AccountKey),
            max_share_per_account,
            is_paused: false,
        }
    }

    pub fn get_info(&self) -> FaucetInfo {
        FaucetInfo::from(&self)
    }

    pub fn shared_balance_of(&self, account_id: AccountId) -> Balance {
        self.accounts.get(&account_id).unwrap_or_else(|| 0)
    }

    pub fn update_max_share(&mut self, max_share: Balance) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only owner can update this field"
        );
        self.max_share_per_account = max_share;
    }

    #[payable]
    pub fn faucet_token(&mut self, amount: U128) -> Promise {
        assert!(
            env::attached_deposit() > 1,
            "Please deposit at least 1 yocto NEAR"
        );
        assert!(!self.is_paused, "Faucet is paused");
        assert!(
            self.available_balance >= amount.0,
            "Not enough token to share"
        );

        let account_id = env::predecessor_account_id();
        let account_balance: Balance = self.accounts.get(&account_id).unwrap_or_else(|| 0);
        assert!(
            account_balance + amount.0 <= self.max_share_per_account,
            "Exceeded maximum amount"
        );

        ext_ft_contract::ft_transfer(
            account_id.clone(),
            amount,
            Some("Faucet token from DVP".to_string()),
            &self.ft_contract_id, // Calling contract (FT contract)
            ONE_YOCTO_NEAR,       // Deposit amount
            FT_TRANSFER_GAS,      // Gas amount
        )
        .then(ext_self::ft_transfer_callback(
            amount,
            account_id.clone(),
            &env::current_account_id(), // Calling contract (Faucet contract)
            0,                          // Deposit amount
            FAUCET_CALLBACK_GAS,        // Gas amount
        ))
    }

    // After transfering successfully
    #[private]
    pub fn ft_transfer_callback(&mut self, amount: U128, account_id: AccountId) {
        let mut account_balance = self.accounts.get(&account_id).unwrap_or_else(|| 0);
        if account_balance == 0 {
            self.total_share_account += 1;
        }
        account_balance += amount.0;

        self.accounts.insert(&account_id, &account_balance);
        self.current_shared_balance += amount.0;
        self.available_balance -= amount.0;
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for FaucetContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            sender_id, self.owner_id,
            "Only owner can transfer to the faucet"
        );
        assert_eq!(
            env::predecessor_account_id(),
            self.ft_contract_id,
            "Only accept token from the correct FT"
        );

        self.available_balance += amount.0;
        env::log(
            format!(
                "Receive {} tokens from {} with message \"{}\"",
                &amount.0, &sender_id, &msg
            )
            .as_bytes(),
        );

        PromiseOrValue::Value(U128(0))
    }
}

use near_sdk::{collections::LookupMap, env::{self, account_balance}, json_types::U64, near_bindgen, Balance, ext_contract, PromiseOrValue, AccountId};

use StorageKey;
mod StorageKey;

const ONE_YOCTO_NEAR: Balance = 1;
pub const FT_TRANSFER_GAS: Gas = 10_000_000_000_000;
pub const FAUCET_CALLBACK_GAS: Gas = 10_000_000_000_000;


// ====================================== Faucet info =================================

// To cast to JSON
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FaucetInfo {
    pub current_shared_balance: Balance,
    pub available_balance: Balance,
    pub total_share_account: u64,
    pub max_share_per_account: Balance,
    pub is_paused: bool,
}

impl FaucetInfo {
    pub fn from(contract: FaucetContract) -> FaucetInfo {
        FaucetInfo {
            current_shared_balance: contract.current_shared_balance,
            available_balance: contract.available_balance,
            total_share_account: contract.total_share_account,
            max_share_per_account: contract.max_share_per_account,
            is_paused: contract.is_paused,
        }
    }
}

// ====================================== External interface ====================================

// Token transfering method of NEP141
#[ext_contract(ext_ft_contract)]
pub trait FungibleTokenCore {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: Balance, memo: Option<String>);
}

// For callback after transfer token
#[ext_contract(ext_self)]
pub trait ExtFaucetContract {
    fn ft_transfer_callback(&mut self, amount: Balance, account_id: AccountId);
}

// For transfering balance share from owner
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: Balance, msg: String) -> PromiseOrValue<u128>;
}

// ======================================== Faucet contract =====================================

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FaucetContract {
    pub owner_id: AccountId,                        // Contract owner (for transfer tokens)
    pub ft_contract_id: AccountId,                  // FT contract account
    pub current_shared_balance: Balance,            // Shared tokens
    pub available_balance: Balance,                 // Current available to faucet
    pub total_share_account: U64,                   // Number of sharing accounts
    pub accounts: LookupMap<AccountId, Balance>,    // Balance of account which was achieved (no storage deposit)
    pub max_share_per_account: Balance,             // Max balance fauceting for each account
    pub is_paused: bool,                            // Contract status
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
            accounts: LookupMap::new(StorageKey.AccountKey),
            max_share_per_account,
            is_paused: false,
        }
    }

    pub fn get_info(&self) -> FaucetInfo {
        FaucetInfo::from(self)
    }

    pub fn shared_balance_of(&self, account_id: AccountId) -> Balance {
        self.accounts.get(&account_id).unwrap_or_else(|| 0)
    }

    pub fn update_max_share(&self, max_share: Balance) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only owner can update this field");
        self.max_share_per_account = max_share;
    }

    #[payable]
    pub fn faucet_token(&mut self, amount: Balance) -> Promise {
        assert!(env::attached_deposit() > 1, "Please deposit at least 1 yocto NEAR");
        assert!(!self.is_paused, "Faucet is paused");
        assert!(self.available_balance >= amount, "Not enough token to share");
    
        let account_id = env::predecessor_account_id();
        let account_balance = self.accounts.get(&account_id).unwrap_or_else(0);
        assert!(account_balance + amount <= self.max_share_per_account, "Exceeded maximum amount");

        ext_ft_contract::ft_transfer(
            account_id.clone(),
            amount,
            Some("Faucet token from DVP".to_string()),
            &self.ft_contract_id,                       // Calling contract (FT contract)
            ONE_YOCTO_NEAR,                             // Deposit amount
            FT_TRANSFER_GAS                             // Gas amount
        ).then(ext_self::ft_transfer_callback(
            amount,
            account_id.clone(),
            env::current_account_id(),                  // Calling contract (Faucet contract)
            0,                                          // Deposit amount
            FAUCET_CALLBACK_GAS                         // Gas amount
        ))
    }

    // After transfering successfully
    #[private]
    pub fn ft_transfer_callback(&mut self, amount: Balance, account_id: AccountId) {
        let mut account_balance = self.accounts.get(&account_id).unwrap_or_else(|| 0);
        if (account_balance == 0) {
            self.total_share_account += 1;
        }
        account_balance += amount;

        self.accounts.insert(&account_id, &account_balance);
        self.current_shared_balance += amount;
        self.available_balance -= amount;
    }
}

impl FungibleTokenReceiver for FaucetContract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: Balance, msg: String) -> PromiseOrValue<u128> {
        assert_eq!(sender_id, self.owner_id, "Only owner can transfer to the faucet");
        assert_eq!(env::predecessor_account_id(), self.ft_contract_id, "Only accept token from the correct FT");

        self.available_balance += amount;

        PromiseOrValue::Value(0)
    }
}
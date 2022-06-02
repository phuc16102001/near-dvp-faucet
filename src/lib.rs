use near_sdk::{collections::LookupMap, env, json_types::U64, near_bindgen, Balance, ext_contract, PromiseOrValue};

use StorageKey;
mod StorageKey;

// ====================================== Faucet info =================================

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
    pub owner_id: AccountId,
    pub ft_contract_id: AccountId,
    pub current_shared_balance: Balance,
    pub available_balance: Balance,
    pub total_share_account: U64,
    pub accounts: LookupMap<AccountId, Balance>,
    pub max_share_per_account: Balance,
    pub is_paused: bool,
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
}

impl FungibleTokenReceiver for FaucetContract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: Balance, msg: String) -> PromiseOrValue<u128> {
        assert_eq!(sender_id, self.owner_id, "Only owner can transfer to the faucet");
        assert_eq!(env::predecessor_account_id(), self.ft_contract_id, "Only accept token from the correct FT");

        self.available_balance += amount;

        PromiseOrValue::Value(0)
    }
}
use near_sdk::{collections::LookupMap, env, json_types::U64, near_bindgen, Balance};

use StorageKey;

mod StorageKey;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FaucetContract {
    pub owner_id: AccountId,
    pub ft_contract_id: AccountId,
    pub current_shared_balance: Balance,
    pub total_shared_balance: Balance,
    pub total_share_account: u64,
    pub accounts: LookupMap<AccountId, Balance>,
    pub max_share_per_account: Balance,
    pub is_paused: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct FaucetContractJson {
    pub current_shared_balance: Balance,
    pub total_shared_balance: Balance,
    pub total_share_account: u64,
    pub max_share_per_account: Balance,
    pub is_paused: bool,
}

impl FaucetInfo {
    pub fn from(contract: FaucetContract) -> FaucetInfo {
        FaucetInfo {
            current_shared_balance: contract.current_shared_balance,
            total_shared_balance: contract.total_shared_balance,
            total_share_account: contract.total_share_account,
            max_share_per_account: contract.max_share_per_account,
            is_paused: contract.is_paused,
        }
    }
}

#[near_bindgen]
impl FaucetContract {
    #[init]
    fn new(owner_id: AccountId, ft_contract_id: AccountId, max_share_per_account: Balance) -> Self {
        FaucetContract {
            owner_id,
            ft_contract_id,
            current_shared_balance: 0,
            total_shared_balance: 0,
            total_share_account: 0,
            accounts: LookupMap::new(StorageKey.AccountKey),
            max_share_per_account,
            is_paused: false,
        }
    }
}

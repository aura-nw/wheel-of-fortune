use crate::state::{Config, WheelReward};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Timestamp};
use nois::NoisCallback;

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    // length must be less than 64 character
    pub wheel_name: String,
    // must be hex string and has length 64
    pub random_seed: String,
    // must greater than 0
    pub max_spins_per_address: u32,
    pub is_public: bool,
    pub is_advanced_randomness: bool,
    // bench32 string address
    pub nois_proxy: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    // admin methods
    AddWhitelist {
        addresses: Vec<String>,
    },

    RemoveWhitelist {
        addresses: Vec<String>,
    },

    RemoveReward {
        slot: u32,
    },

    AddReward {
        reward: WheelReward,
    },

    ActivateWheel {
        price: Coin,
        start_time: Option<Timestamp>,
        end_time: Timestamp,
        shuffle: Option<bool>,
    },

    Withdraw {
        slot: u32,
        recipient: Option<String>,
    },

    WithdrawCoin {
        denom: String,
        recipient: Option<String>,
    },

    // user methods
    Spin {
        number: Option<u32>,
    },

    ClaimReward {
        rewards: Vec<u32>,
    },

    // nois callback
    NoisReceive {
        callback: NoisCallback,
    },
}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Vec<WheelReward>)]
    GetWheelRewards {},

    #[returns(Option<Vec<(bool, WheelReward)>>)]
    GetPlayerRewards { address: String },

    #[returns(AllEarnedPrizeResponse)]
    AllEarnedPrize {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    #[returns(Option<u32>)]
    GetPlayerSpinned { address: String },

    #[returns(Config)]
    GetWheelConfig {},

    #[returns(Option<bool>)]
    Spinnable { address: String },

    #[returns(Option<Vec<WhiteListResponse>>)]
    GetWhiteList {},
}

#[cw_serde]
pub struct WhiteListResponse {
    pub addresses: Vec<Addr>,
}

#[cw_serde]
pub struct EarnedPrizes {
    pub address: String,
    pub rewards: Vec<(bool, WheelReward)>,
}

#[cw_serde]
pub struct AllEarnedPrizeResponse {
    pub all_earned_prizes: Vec<EarnedPrizes>,
}

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}

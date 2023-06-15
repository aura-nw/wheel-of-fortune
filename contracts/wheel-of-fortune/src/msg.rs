use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Timestamp;
use crate::state::{WheelReward, UserFee, Config};
use nois::NoisCallback;


/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub wheel_name: String,
    pub random_seed: String,
    pub max_spins_per_address: u32,
    pub is_public: bool,
    pub is_advanced_randomness: bool,
    pub nois_proxy: String,
}

/* 
'{"wheel_name":"test","random_seed":"12345678","max_spins_per_address":5,"is_public":true,"is_advanced_randomness":false,"nois_proxy":"aura159mt7ryhxd9g07fjw5lpreqnv8yzuf72vh22zg","fee":{"denom":"uaura","spin_price":"100","nois_fee":"300"}}'
*/

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    // admin methods
    AddWhitelist {
        addresses: Vec<String>
    },

    RemoveWhitelist {
        addresses: Vec<String>
    },

    RemoveReward {
        slot: usize
    },

    AddReward {
        reward: WheelReward
    },

    ActivateWheel {
        fee: UserFee,
        start_time: Option<Timestamp>,
        end_time: Timestamp
    },

    Withdraw {
        recipient: Option<String>,
        denom: String,
    },

    WithdrawNft {
        recipient: Option<String>,
        collection: String,
        token_ids: Vec<String>
    },

    WithdrawToken {
        recipient: Option<String>,
        token_address: String
    },

    // user methods
    Spin {
        number: Option<u32>
    },

    ClaimReward {
        rewards: Vec<usize>
    },

    // nois callback
    NoisReceive {
        callback: NoisCallback 
    }
}


/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {

    #[returns(Vec<WheelReward>)]
    GetWheelRewards{},

    #[returns(Option<Vec<(bool, WheelReward)>>)]
    GetPlayerRewards{address: String},

    #[returns(Option<u32>)]
    GetPlayerSpinned{address: String},

    #[returns(Config)]
    GetWheelConfig{},
}

// We define a custom struct for each query response
// #[cw_serde]
// pub struct YourQueryResponse {}

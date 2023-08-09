use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Addr, Timestamp, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct UserFee {
    pub denom: String,
    pub spin_price: Uint128,
    pub nois_fee: Uint128
}

impl UserFee {
    pub fn default() -> UserFee {
        UserFee { 
            denom: "".to_string(), 
            spin_price: Uint128::zero(), 
            nois_fee: Uint128::zero() 
        }
    }
}

#[cw_serde]
pub struct Config {
    pub wheel_name: String,
    pub max_spins_per_address: u32,
    pub is_public: bool,
    pub is_advanced_randomness: bool,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub nois_proxy: Addr,
    pub fee: UserFee
}
pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct AdminConfig {
    pub admin: Addr,
    pub activate: bool,
}
pub const ADMIN_CONFIG: Item<AdminConfig> = Item::new("admin config");

#[cw_serde]
pub struct CollectionReward {
    pub label: String,
    pub collection_address: String,
    pub token_ids: Vec<String>,
    pub id: u32
}

#[cw_serde]
pub struct CoinReward {
    pub label: String,
    pub coin: Coin,
    pub number: u32,
    pub id: u32
}


#[cw_serde]
pub struct TextReward {
    pub label: String,
    pub number: u32,
    pub id: u32
}

#[cw_serde]
pub struct TokenReward {
    pub label: String,
    pub token_address: String,
    pub amount: Uint128,
    pub number: u32,
    pub id: u32
}

#[cw_serde]
pub enum WheelReward {
    NftCollection(CollectionReward),
    FungibleToken(TokenReward),
    Coin(CoinReward),
    Text(TextReward)
}


impl WheelReward {
    pub fn get_supply(&self) -> u32 {
        match (*self).clone() {
            Self::NftCollection(colecttion) => {
                return colecttion.token_ids.len() as u32;
            }
            Self::FungibleToken(token) => {
                return token.number;
            }
            Self::Coin(coin) => {
                return coin.number;
            }
            Self::Text(text) => {
                return text.number;
            }
        }
    }
}
pub const WHEEL_REWARDS: Item<(u32, Vec<WheelReward>)> = Item::new("wheel rewards");

#[cw_serde]
pub struct RandomJob {
    pub player: Addr,
    pub spins: u32,
}
pub const RANDOM_JOBS: Map<String, RandomJob> = Map::new("random jobs");

pub const RANDOM_SEED: Item<[u8; 32]> = Item::new("random seed");

pub const WHITELIST: Map<Addr, u32> = Map::new("whitelist");

pub const SPINS_RESULT: Map<Addr, Vec<(bool, WheelReward)>> = Map::new("spins result");
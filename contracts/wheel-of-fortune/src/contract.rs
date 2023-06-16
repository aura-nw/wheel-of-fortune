#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, MessageInfo, Response, ensure_eq, BankMsg, BankQuery, BalanceResponse,
    StdResult, Storage, Addr, Timestamp, WasmMsg, to_binary, CosmosMsg, Uint128, Coin, coins, QueryRequest
};
use cw2::set_contract_version;

use cw721_base::{ExecuteMsg as CW721ExecuteMsg, Extension as CW721Extension};

use cw20::{Cw20ExecuteMsg, BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, CONFIG, AdminConfig, ADMIN_CONFIG, RANDOM_SEED, WHITELIST, CollectionReward, CoinReward,
    WheelReward, WHEEL_REWARDS, TokenReward, RandomJob, RANDOM_JOBS, TextReward, SPINS_RESULT, UserFee
};

use nois::{
    randomness_from_str, NoisCallback, select_from_weighted,
    ProxyExecuteMsg, sub_randomness_with_key
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:wheel-of-fortune";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_TEXT_LENGTH: usize = 64;
const MAX_SPINS_PER_TURN: u32 = 10;
const DEFAULT_ACTIVATE: bool = false;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if msg.wheel_name.len() > MAX_TEXT_LENGTH {
        return Err(ContractError::TextTooLong {});
    }
    
    if msg.max_spins_per_address == 0 {
        return Err(ContractError::CustomError {val: "the maximum number of spins must be greater than 0".to_string()});
    }

    let nois_proxy = deps.api.addr_validate(&msg.nois_proxy)?;

    let config = Config { 
        wheel_name: msg.wheel_name, 
        max_spins_per_address: msg.max_spins_per_address, 
        is_public: msg.is_public, 
        is_advanced_randomness: msg.is_advanced_randomness,
        start_time: None,
        end_time: None,
        fee: UserFee::default(),
        nois_proxy
    };
    CONFIG.save(deps.storage, &config)?;

    let admin_config = AdminConfig { 
        admin: info.sender.clone(), 
        activate: DEFAULT_ACTIVATE 
    };
    ADMIN_CONFIG.save(deps.storage, &admin_config)?;

    // save the init RANDOM_SEED to the storage
    let randomness = randomness_from_str(msg.random_seed).unwrap();
    RANDOM_SEED.save(deps.storage, &randomness)?;

    WHEEL_REWARDS.save(deps.storage, &Vec::new())?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}


/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // admin methods
        ExecuteMsg::AddWhitelist { addresses } => add_whitelist(deps, info, addresses),
        ExecuteMsg::RemoveWhitelist { addresses } => remove_whitelist(deps, info, addresses),
        ExecuteMsg::AddReward { reward } => add_reward(deps, env, info, reward),
        ExecuteMsg::RemoveReward { slot } => remove_reward(deps, info, slot),
        ExecuteMsg::ActivateWheel { fee, start_time, end_time } => activate_wheel(deps, env, info, fee, start_time, end_time),
        ExecuteMsg::Withdraw { recipient, denom } => withdraw(deps, env, info, recipient, denom),
        ExecuteMsg::WithdrawNft { recipient, collection, token_ids } => withdraw_nft(deps, env, info, recipient, collection, token_ids),
        ExecuteMsg::WithdrawToken { recipient, token_address } => withdraw_token(deps, env, info, recipient, token_address),

        // user methods
        ExecuteMsg::Spin { number } => spin(deps, env, info, number),
        ExecuteMsg::ClaimReward { rewards } => claim_reward(deps, env, info, rewards),
        
        //nois callback
        ExecuteMsg::NoisReceive { callback } => nois_receive(deps, env, info, callback)
    }
}

pub fn add_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>
) -> Result<Response, ContractError> {

    // check if wheel is not activated and sender is contract admin
    is_not_activate_and_owned(deps.storage, info.sender)?;

    for address in addresses {
        let addr = Addr::unchecked(address.clone());

        if !WHITELIST.has(deps.storage, addr.clone()) {
            WHITELIST.save(
                deps.storage,
                addr.clone(),
                &0,
            )?;

            SPINS_RESULT.save(deps.storage, addr, &Vec::new())?;
        }
    }

    Ok(Response::new().add_attribute("action", "add_whitelist"))
}

pub fn remove_whitelist(
    deps: DepsMut,
    info: MessageInfo,
    addresses: Vec<String>
) -> Result<Response, ContractError> {

    // check if wheel is not activated and sender is contract admin
    is_not_activate_and_owned(deps.storage, info.sender)?;

    for address in addresses {
        WHITELIST.remove(deps.storage, Addr::unchecked(address));
    }

    Ok(Response::new().add_attribute("action", "remove_whitelist"))
}

fn add_collection_reward(
    wheel_rewards: &mut Vec<WheelReward>,
    msgs: &mut Vec<CosmosMsg>,
    recipient: String,
    collection: CollectionReward
) -> Result<(), ContractError> {
    
    if collection.token_ids.len() > (u32::MAX as usize) {
        return Err(ContractError::TooManyNfts {})
    }

    if collection.label.len() > MAX_TEXT_LENGTH {
        return Err(ContractError::TextTooLong {});
    }

    transfer_nft_msgs(
        msgs, 
        recipient, 
        collection.collection_address.clone(), 
        collection.token_ids.clone()
    )?;

    wheel_rewards.push(WheelReward::NftCollection(collection));

    Ok(())
}

fn add_token_reward(
    wheel_rewards: &mut Vec<WheelReward>,
    msgs: &mut Vec<CosmosMsg>,
    recipient: String,
    token: TokenReward
) -> Result<(), ContractError> {

    if token.label.len() > MAX_TEXT_LENGTH {
        return Err(ContractError::TextTooLong {});
    }

    let total_amount = token.amount.checked_mul(Uint128::from(token.number as u128)).unwrap();

    transfer_token_msg(
        msgs, 
        recipient, 
        token.token_address.clone(), 
        total_amount
    )?;

    wheel_rewards.push(WheelReward::FungibleToken(token));

    Ok(())
}

fn add_coin_reward(
    wheel_rewards: &mut Vec<WheelReward>,
    funds: Vec<Coin>,
    coin: CoinReward
) -> Result<(), ContractError> {

    if coin.label.len() > MAX_TEXT_LENGTH {
        return Err(ContractError::TextTooLong {});
    }

    let total_amount = coin.coin.amount.checked_mul(Uint128::from(coin.number as u128)).unwrap();
        
    if has_coin(funds, coin.coin.denom.clone(), total_amount) {
        return Err(ContractError::InsufficentFund {});
    }

    wheel_rewards.push(WheelReward::Coin(coin));

    Ok(())
}

fn add_text_reward(
    wheel_rewards: &mut Vec<WheelReward>,
    text: TextReward
) -> Result<(), ContractError> {
    
    if text.label.len() > MAX_TEXT_LENGTH {
        return Err(ContractError::TextTooLong {});
    }

    wheel_rewards.push(WheelReward::Text(text));

    Ok(())
}

pub fn add_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    reward: WheelReward
) -> Result<Response, ContractError> {

    // check if wheel is not activated and sender is contract admin
    is_not_activate_and_owned(deps.storage, info.sender)?;

    // list rewards of the wheel
    let mut wheel_rewards = WHEEL_REWARDS.load(deps.storage)?;

    // maybe useless check
    if wheel_rewards.len() >= (u32::MAX as usize){
        return Err(ContractError::TooManyRewards {});
    }

    let mut msgs: Vec<CosmosMsg> = Vec::new();

    match reward {
        WheelReward::NftCollection(collection) => {
            // add collection to wheel rewards list
            add_collection_reward(wheel_rewards.as_mut(), msgs.as_mut(), env.contract.address.to_string(), collection)?;
        }
        WheelReward::FungibleToken(token) => {
            // add token to wheel rewards list
            add_token_reward(wheel_rewards.as_mut(), msgs.as_mut(), env.contract.address.to_string(), token)?;
        }
        WheelReward::Coin(coin) => {
            // add coint to wheel rewards list
            add_coin_reward(wheel_rewards.as_mut(), info.funds, coin)?;
        }
        WheelReward::Text(text) => {
            // add text to wheel rewards list
            add_text_reward(wheel_rewards.as_mut(), text)?;
        }
    }

    WHEEL_REWARDS.save(deps.storage, &wheel_rewards)?;

    if msgs.len() > 0 {
        Ok(Response::new().add_attribute("action", "add_rewards")
            .add_messages(msgs))
    }else{
        Ok(Response::new().add_attribute("action", "add_rewards"))
    }
}

fn remove_reward(
    deps: DepsMut,
    info: MessageInfo,
    slot: u32
) -> Result<Response, ContractError> {

    // check if wheel is not activated and sender is contract admin
    is_not_activate_and_owned(deps.storage, info.sender.clone())?;

    // list rewards of the wheel
    let mut wheel_rewards = WHEEL_REWARDS.load(deps.storage)?;

    // slot out of range
    if (slot as usize) >= wheel_rewards.len() {
        return Err(ContractError::InvalidSlotReward {});
    } 
    
    // get and remove reward at slot
    let reward = wheel_rewards.remove(slot as usize);

    let mut msgs: Vec<CosmosMsg> = Vec::new();

    withdraw_reward_msgs(reward, info.sender.to_string(), msgs.as_mut())?;

    if msgs.len() > 0 {
        return Ok(Response::new().add_attribute("action", "remove_reward")
            .add_attribute("slot", slot.to_string())
            .add_messages(msgs));
    }else {
        return Ok(Response::new().add_attribute("action", "remove_reward")
            .add_attribute("slot", slot.to_string()))
    }
}

pub fn activate_wheel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fee: UserFee,
    start_time: Option<Timestamp>,
    end_time: Timestamp
) -> Result<Response, ContractError> {

    // check if wheel is not activated and sender is contract admin
    is_not_activate_and_owned(deps.storage, info.sender)?;

    if let Some(start_time) = start_time {
        if start_time >= end_time {
            return Err(ContractError::InvalidTimeSetting {})
        }
    }

    if end_time <= env.block.time {
        return Err(ContractError::WheelEnded {});
    }

    let mut admin_config: AdminConfig = ADMIN_CONFIG.load(deps.storage)?;

    admin_config.activate = true;
    ADMIN_CONFIG.save(deps.storage, &admin_config)?;

    let mut config = CONFIG.load(deps.storage)?;
    config.fee = fee;
    config.start_time = start_time;
    config.end_time = Some(end_time);
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "activate_wheel"))
}

pub fn spin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    number: Option<u32>
) -> Result<Response, ContractError> {
    
    let admin_config = ADMIN_CONFIG.load(deps.storage)?;
    if !admin_config.activate {
        return Err(ContractError::WheelNotActivated {});
    } 

    let config = CONFIG.load(deps.storage)?;

    let spins = number.unwrap_or(1);
    if spins == 0 || spins > MAX_SPINS_PER_TURN {
        return Err(ContractError::InvalidNumberSpins {});
    }

    let spinned_result = WHITELIST.may_load(deps.storage, info.sender.clone())?;

    if !config.is_public && spinned_result.is_none() {
        return Err(ContractError::Unauthorized {});
    } 
    
    if let Some(start_time) = config.start_time {
        if start_time > env.block.time {
            return Err(ContractError::WheelNotStarted{})
        }
    }

    if config.end_time.unwrap() < env.block.time {
        return Err(ContractError::WheelEnded{});
    }

    if spinned_result.is_none() {
        SPINS_RESULT.save(deps.storage, info.sender.clone(), &Vec::new())?;
    }

    let spinned = spinned_result.unwrap_or(0);

    let mut total_amount = config.fee.spin_price.checked_mul(Uint128::from(spins as u128)).unwrap();

    // if contract using advanced randomness mode, player must pay for nois randomness request
    if config.is_advanced_randomness {
        total_amount = total_amount.checked_add(config.fee.nois_fee).unwrap();
    }

    if has_coin(info.funds, config.fee.denom.clone(), total_amount) {
        return Err(ContractError::InsufficentFund {});
    }

    if spins > (config.max_spins_per_address - spinned) {
        return Err(ContractError::CustomError {
            val: format!("Too many spins request: {} left", config.max_spins_per_address - spinned)
        });
    }

    WHITELIST.save(deps.storage, info.sender.clone(), &(spinned + spins))?;

    if config.is_advanced_randomness {

        let job_id = format!("{}/{}", info.sender, spinned);

        let msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.nois_proxy.into(),
            msg: to_binary(&ProxyExecuteMsg::GetNextRandomness { 
                            job_id: job_id.clone() })?,
            funds: coins(config.fee.nois_fee.u128(), config.fee.denom),
        });

        let random_job = RandomJob { 
            player: info.sender.clone(), 
            spins 
        };

        RANDOM_JOBS.save(deps.storage, job_id, &random_job)?;

        return Ok(Response::new().add_attribute("action", "spin")
            .add_attribute("sender", info.sender)
            .add_attribute("spins", spins.to_string())
            .add_message(msg));
    }else {

        // load RANDOM_SEED from the storage
        let random_seed = RANDOM_SEED.load(deps.storage)?;

        // init a key for the random provider from the msg.sender and current time
        let key = format!("{}{}", info.sender, env.block.time);

        // select rewards for player
        let new_random_seed = select_wheel_rewards(deps.storage, info.sender.clone(), random_seed, key, spins)?;

        RANDOM_SEED.save(deps.storage, &new_random_seed)?;

        return Ok(Response::new().add_attribute("action", "spin")
            .add_attribute("sender", info.sender)
            .add_attribute("spins", spins.to_string()));
    }
}

pub fn claim_reward(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    rewards: Vec<u32>
) -> Result<Response, ContractError> {

    let admin_config = ADMIN_CONFIG.load(deps.storage)?;
    if !admin_config.activate {
        return Err(ContractError::WheelNotActivated {});
    }

    let mut spins_result = 
        if let Some(result) =  SPINS_RESULT.may_load(deps.storage, info.sender.clone())? {
            result
        }else{
            return Err(ContractError::PlayerNotFound {});
        };

    let mut msgs: Vec<CosmosMsg> = Vec::new();

    for idx in rewards {
        if let Some((is_claimed, reward)) = spins_result.get(idx as usize){
            if !is_claimed {

                withdraw_reward_msgs(reward.to_owned(), info.sender.to_string(), msgs.as_mut())?;

                spins_result[idx as usize].0 = true;       
            }
        }
    }
    
    // update player reward
    SPINS_RESULT.save(deps.storage, info.sender.clone(), &spins_result)?;

    if msgs.len() > 0 {
        Ok(Response::new().add_attribute("action", "claim_reward")
            .add_attribute("sender", info.sender)
            .add_messages(msgs))
    }else {
        Ok(Response::new().add_attribute("action", "claim_reward")
            .add_attribute("sender", info.sender))
    }
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    denom: String
) -> Result<Response, ContractError> {

    // check if wheel is activated and sender is contract admin
    is_activate_and_owned(deps.storage, info.sender.clone())?;
    
    let config = CONFIG.load(deps.storage)?;
    if config.end_time.unwrap() > env.block.time {
        return Err(ContractError::WheelNotEnded {});
    }

    // get the balance of contract in bank
    let contract_balance: BalanceResponse =
        deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
            address: env.contract.address.to_string(),
            denom: denom.clone(),
        }))?;
    
    if contract_balance.amount.amount == Uint128::zero() {
        return Err(ContractError::InsufficentFund {});
    }
    
    let recipient = recipient.unwrap_or(info.sender.to_string());
    deps.api.addr_validate(&recipient)?;

    // with draw token
    let send_msg = send_coin_msg(recipient.clone(), vec![contract_balance.amount])?;

    Ok(Response::new().add_attribute("action", "withdraw")
        .add_attribute("denom", denom)
        .add_attribute("receiver", recipient)
        .add_message(send_msg))
}

pub fn withdraw_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    collection_address: String,
    token_ids: Vec<String>
) -> Result<Response, ContractError> {

    // check if wheel is activated and sender is contract admin
    is_activate_and_owned(deps.storage, info.sender.clone())?;
    
    let config = CONFIG.load(deps.storage)?;
    if config.end_time.unwrap() > env.block.time {
        return Err(ContractError::WheelNotEnded {});
    }

    let recipient = recipient.unwrap_or(info.sender.to_string());

    deps.api.addr_validate(&recipient)?;
    deps.api.addr_validate(&collection_address)?;

    let mut msgs: Vec<CosmosMsg> = Vec::new();
    transfer_nft_msgs(msgs.as_mut(), recipient.clone(), collection_address, token_ids)?;

    Ok(Response::new().add_attribute("action", "withdraw_nft")
    .add_attribute("receiver", recipient)
    .add_messages(msgs))
}

pub fn withdraw_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Option<String>,
    token_address: String
) -> Result<Response, ContractError> {

    // check if wheel is activated and sender is contract admin
    is_activate_and_owned(deps.storage, info.sender.clone())?;
    
    let config = CONFIG.load(deps.storage)?;
    if config.end_time.unwrap() > env.block.time {
        return Err(ContractError::WheelNotEnded {});
    }

    let recipient = recipient.unwrap_or(info.sender.to_string());

    deps.api.addr_validate(&recipient)?;
    deps.api.addr_validate(&token_address)?;
    
    // get the token balance of contract
    let contract_balance: Cw20BalanceResponse =
            deps.querier.query_wasm_smart(
                token_address.clone(),
                &Cw20QueryMsg::Balance { 
                    address: env.contract.address.to_string()
                }
            )?;
    
    if contract_balance.balance == Uint128::zero() {
        return Err(ContractError::InsufficentFund {});
    }

    let mut msgs: Vec<CosmosMsg> = Vec::new();
    transfer_token_msg(msgs.as_mut(), recipient.clone(), token_address, contract_balance.balance)?;

    Ok(Response::new().add_attribute("action", "withdraw_token")
    .add_attribute("receiver", recipient)
    .add_messages(msgs))
}

pub fn nois_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    callback: NoisCallback
) -> Result<Response, ContractError> {

    let config: Config = CONFIG.load(deps.storage)?;

    ensure_eq!(info.sender, config.nois_proxy, ContractError::Unauthorized{});

    let job_id = callback.job_id;
    let randomness: [u8; 32] = callback
        .randomness
        .to_array()
        .map_err(|_| ContractError::InvalidRandomness{})?;

    let random_job: RandomJob = 
        if let Some(job) = RANDOM_JOBS.may_load(deps.storage, job_id)? {
            job
        }else{
            return Err(ContractError::RandomJobNotFound {});
        };

    let key = format!("{}", env.block.time);

    let _ = select_wheel_rewards(deps.storage, random_job.player, randomness, key, random_job.spins)?;

    Ok(Response::new().add_attribute("action", "nois_receive"))
}

fn has_coin(
    funds: Vec<Coin>,
    denom: String,
    amount: Uint128
) -> bool {
    if funds.len() != 1 || 
        funds[0].denom != denom || 
         funds[0].amount != amount {
            return false;
    }
    true
}

fn is_not_activate_and_owned(storage: &dyn Storage, sender: Addr) -> Result<(), ContractError> {
    let admin_config = ADMIN_CONFIG.load(storage)?;
    if admin_config.admin != sender {
        return Err(ContractError::Unauthorized {});
    }

    if admin_config.activate {
        return Err(ContractError::WheelActivated {});
    }

    Ok(())
}

fn is_activate_and_owned(storage: &dyn Storage, sender: Addr) -> Result<(), ContractError> {
    let admin_config = ADMIN_CONFIG.load(storage)?;
    if admin_config.admin != sender {
        return Err(ContractError::Unauthorized {});
    }

    if !admin_config.activate {
        return Err(ContractError::WheelNotActivated {});
    }

    Ok(())
}

fn select_wheel_rewards(
    storage: &mut dyn Storage,
    player: Addr,
    random_seed: [u8; 32],
    key: String,
    spins: u32
) -> Result<[u8; 32], ContractError> {

    let mut wheel_rewards = WHEEL_REWARDS.load(storage)?;

    let mut spins_result = SPINS_RESULT.load(storage, player.clone())?;

    let mut list_weighted: Vec<(usize, u32)> = Vec::new();
    for idx in 0..wheel_rewards.len() {

        let reward_supply =  wheel_rewards[idx].get_supply();
        
        list_weighted.push((idx, reward_supply));
    }

    // define random provider from the random_seed
    let mut provider = sub_randomness_with_key(random_seed, key);

    let mut randomness = [0u8; 32];

    for _ in 0..spins {
        // random a new randomness
        randomness = provider.provide();

        let reward_idx: usize = select_from_weighted(randomness, &list_weighted).unwrap();

        // update weighted
        list_weighted[reward_idx].1 -= 1;

        match wheel_rewards[reward_idx].clone() {
            WheelReward::NftCollection(mut collection) => {
                let reward = WheelReward::NftCollection(CollectionReward { 
                    label: collection.label.clone(), 
                    collection_address: collection.collection_address.clone(), 
                    token_ids: vec![collection.token_ids.pop().unwrap()] 
                });
                
                wheel_rewards[reward_idx] = WheelReward::NftCollection(collection);

                spins_result.push((false, reward));
            }

            WheelReward::FungibleToken(mut token) => {
                let reward = WheelReward::FungibleToken(TokenReward { 
                    label: token.label.clone(), 
                    token_address: token.token_address.clone(), 
                    amount: token.amount, 
                    number: 1 
                });

                token.number -= 1;
                
                wheel_rewards[reward_idx] = WheelReward::FungibleToken(token);

                spins_result.push((false, reward));
            }

            WheelReward::Coin(mut coin) => {
                let reward = WheelReward::Coin(CoinReward { 
                    label: coin.label.clone(), 
                    coin: coin.coin.clone(), 
                    number: 1 
                });

                coin.number -= 1;

                wheel_rewards[reward_idx] = WheelReward::Coin(coin);

                spins_result.push((false, reward));
            }

            WheelReward::Text(mut text) => {
                let reward = WheelReward::Text(TextReward { 
                    label: text.label.clone(), 
                    number: 1 
                });

                text.number -= 1;
                
                wheel_rewards[reward_idx] = WheelReward::Text(text);

                spins_result.push((false, reward));
            }
        }
    } 
     
    SPINS_RESULT.save(storage, player, &spins_result)?;

    Ok(randomness)
}

fn withdraw_reward_msgs(
    reward: WheelReward,
    recipient: String,
    msgs: &mut Vec<CosmosMsg>
) -> Result<(), ContractError> {

    match reward {
        WheelReward::NftCollection(collection) => {
            // create msgs for transfering NFTs to recipient
            transfer_nft_msgs(msgs, recipient, collection.collection_address, collection.token_ids)?;
        }
        WheelReward::FungibleToken(token) => {
            // create msg for transfering fungible token to recipient
            transfer_token_msg(msgs, recipient, token.token_address, token.amount)?;
        }
        WheelReward::Coin(coin) => {
            // send token to recipient
            let send_msg = send_coin_msg(recipient, vec![coin.coin])?;
            msgs.push(send_msg);
        }
        WheelReward::Text(_text) => {}
    }

    Ok(())
}

fn transfer_nft_msgs(
    msgs: &mut Vec<CosmosMsg>,
    recipient: String,
    contract_addr: String,
    token_ids: Vec<String>
) -> Result<(), ContractError> {
    for token_id in token_ids {
        // transfer NFT to this contract
        let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.clone(),
            msg: to_binary(&CW721ExecuteMsg::<CW721Extension,CW721Extension>::TransferNft {
                recipient: recipient.clone(), 
                token_id
            })?,
            funds: vec![],
        });

        msgs.push(transfer_msg);
    }
    Ok(())
}

fn transfer_token_msg(
    msgs: &mut Vec<CosmosMsg>,
    recipient: String,
    contract_addr: String,
    amount: Uint128
) -> Result<(), ContractError> {

    let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr,
        msg: to_binary(&Cw20ExecuteMsg::Transfer { 
            recipient, 
            amount
        })?,
        funds: vec![],
    });

    msgs.push(transfer_msg);

    Ok(())
}

fn send_coin_msg(
    recipient: String,
    amount: Vec<Coin>
) -> Result<CosmosMsg, ContractError> {
    // send coin to recipient
    let send_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
        to_address: recipient,
        amount,
    });
    Ok(send_msg)
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetWheelRewards{} => to_binary(&get_wheel_rewards(deps)?),
        QueryMsg::GetPlayerRewards{address} => to_binary(&get_player_rewards(deps, address)?),
        QueryMsg::GetPlayerSpinned{address} => to_binary(&get_player_spinned(deps, address)?),
        QueryMsg::GetWheelConfig {} => to_binary(&get_wheel_config(deps)?)
    }
}

fn get_wheel_rewards(
    deps: Deps
) -> StdResult<Vec<WheelReward>> {
    WHEEL_REWARDS.load(deps.storage)
}

fn get_player_rewards(
    deps: Deps,
    address: String
) -> StdResult<Option<Vec<(bool, WheelReward)>>> {
    SPINS_RESULT.may_load(deps.storage, Addr::unchecked(address))
}

fn get_player_spinned(
    deps: Deps,
    address: String
) -> StdResult<Option<u32>> {
    WHITELIST.may_load(deps.storage, Addr::unchecked(address))
}

fn get_wheel_config(
    deps: Deps,
) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}



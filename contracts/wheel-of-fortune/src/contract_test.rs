#[cfg(test)]
mod unit_tests {
    use std::str::FromStr;

    use crate::contract::{instantiate, execute};

    use crate::error::ContractError;
    use crate::msg::{InstantiateMsg, ExecuteMsg};
    use crate::state::{ADMIN_CONFIG, AdminConfig, WheelReward, TextReward, CoinReward, TokenReward, CollectionReward, WHEEL_REWARDS};

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        Uint128, OwnedDeps, Env, Response,BlockInfo, ContractInfo, Timestamp, 
        Addr, Coin, coins, to_binary, WasmMsg, CosmosMsg
    };
    use cw20::Cw20ExecuteMsg;
    use cw721_base::{ExecuteMsg as CW721ExecuteMsg, Extension as CW721Extension};

    const CREATOR: &str = "creator";
    const USER: &str = "user";
    const NOIS_PROXY: &str = "nois proxy";

    // SETUP ENVIROMENT

    fn default_setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { 
            wheel_name: "test".to_string(), 
            random_seed: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), 
            max_spins_per_address: 100, 
            is_public: true, 
            is_advanced_randomness: false, 
            nois_proxy: NOIS_PROXY.to_string()
        };

        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        return deps;
    }

    fn env_with_specify(block_time: Timestamp, block_height: u64) -> Env {
        Env {
            block: BlockInfo {
                height: block_height,
                time: block_time,
                chain_id: mock_env().block.chain_id
            },
            contract: ContractInfo {
                address: mock_env().contract.address
            },
            transaction: None,
        }
    }

    /* ============================================================ INSTANTIATE ============================================================ */
    #[test]
    fn instantiate_works() {
        default_setup();
    }

    #[test]
    fn instantiate_fail_with_long_name() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { 
            wheel_name: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
            aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
            aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
            aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), 
            random_seed: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), 
            max_spins_per_address: 100, 
            is_public: true, 
            is_advanced_randomness: false, 
            nois_proxy: NOIS_PROXY.to_string()
        };

        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match res {
            ContractError::TextTooLong {} => {},
            _ => panic!(),
        };
    }

    #[test]
    fn instantiate_fail_with_invalid_maximum_spins() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { 
            wheel_name: "test".to_string(), 
            random_seed: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), 
            max_spins_per_address: 0, // invalid maximum spins 
            is_public: true, 
            is_advanced_randomness: false, 
            nois_proxy: NOIS_PROXY.to_string()
        };

        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match res {
            ContractError::CustomError {val} => {assert_eq!(val, "the maximum number of spins must be greater than 0".to_string())},
            _ => panic!(),
        };
    }

    #[test]
    fn instantiate_fail_with_invalid_proxy_address() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { 
            wheel_name: "test".to_string(), 
            random_seed: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(), 
            max_spins_per_address: 1, 
            is_public: true, 
            is_advanced_randomness: false, 
            nois_proxy: "".to_string() // Invalid bench32 string address
        };

        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        match res {
            ContractError::InvalidAddress {} => {},
            _ => panic!(),
        };
    }

    #[test]
    #[should_panic]
    fn instantiate_fail_with_invalid_random_seed() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { 
            wheel_name: "test".to_string(), 
            random_seed: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaah".to_string(), // Invalid random seed
            max_spins_per_address: 1, 
            is_public: true, 
            is_advanced_randomness: false,
            nois_proxy: NOIS_PROXY.to_string()
        };

        let info = mock_info(CREATOR, &[]);
        let _ = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    }

    /* ============================================================ AddWhitelist ============================================================ */
    #[test]
    fn add_whitelist_success() {
        let mut deps = default_setup();

        let player_one = "player one".to_string();
        let player_two = "player two".to_string();

        let add_whitelist = ExecuteMsg::AddWhitelist { 
            addresses: vec![player_one, player_two] 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR/*sender is creator*/, &[]), add_whitelist).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "add_whitelist"));
    }

    #[test]
    fn add_whitelist_fail_with_unauthorized() {
        let mut deps = default_setup();

        let player_one = "player one".to_string();
        let player_two = "player two".to_string();

        let add_whitelist = ExecuteMsg::AddWhitelist { 
            addresses: vec![player_one, player_two] 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(USER/*sender is user*/, &[]), add_whitelist).unwrap_err();
        match res {
            ContractError::Unauthorized {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn add_whitelist_success_with_wheel_activated() {
        let mut deps = default_setup();
        
        ADMIN_CONFIG.save(deps.as_mut().storage, &AdminConfig { 
            admin: Addr::unchecked(CREATOR), 
            // set activate to true
            activate: true 
        }).unwrap();

        let player_one = "player one".to_string();
        let player_two = "player two".to_string();

        let add_whitelist = ExecuteMsg::AddWhitelist { 
            addresses: vec![player_one, player_two] 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_whitelist).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "add_whitelist"));
    }

    /* ============================================================ RemoveWhitelist ============================================================ */
    #[test]
    fn remove_whitelist_fail_with_unauthorized() {
        let mut deps = default_setup();

        let player_one = "player one".to_string();
        let player_two = "player two".to_string();

        let remove_whitelist = ExecuteMsg::RemoveWhitelist { 
            addresses: vec![player_one, player_two] 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(USER/*sender is user*/, &[]), remove_whitelist).unwrap_err();
        match res {
            ContractError::Unauthorized {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn remove_whitelist_success_with_wheel_activated() {
        let mut deps = default_setup();

        ADMIN_CONFIG.save(deps.as_mut().storage, &AdminConfig { 
            admin: Addr::unchecked(CREATOR), 
            // set activate to true
            activate: true 
        }).unwrap();

        let player_one = "player one".to_string();
        let player_two = "player two".to_string();

        let remove_whitelist = ExecuteMsg::RemoveWhitelist { 
            addresses: vec![player_one, player_two] 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), remove_whitelist).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "remove_whitelist"));
    }

    /* ============================================================ AddReward ============================================================ */
    // Text
    #[test]
    fn add_text_reward_success() {
        let mut deps = default_setup();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Text(TextReward{
                label: "you lose".to_string(),
                number: 100,
                id: 1,
            }) 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "add_rewards"))
    }

    #[test]
    fn add_text_reward_fail_with_long_label() {
        let mut deps = default_setup();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Text(TextReward{
                label: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                number: 100,
                id: 1,
            }) 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap_err();
        match res {
            ContractError::TextTooLong {} => {}
            _ => panic!()
        }
    }

    // Coin
    #[test]
    fn add_coin_reward_success() {
        let mut deps = default_setup();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Coin(CoinReward{
                label: "100uaura".to_string(),
                coin: Coin { denom: "uaura".to_string(), amount: Uint128::from_str("100").unwrap() },
                number: 100,
                id: 1,
            })
        };

        let res = execute(
            deps.as_mut(), 
            mock_env(), 
            mock_info(CREATOR, &coins(10000u128, "uaura".to_string())/* deposit funds */), 
            add_reward
        ).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "add_rewards"));
    }

    #[test]
    fn add_coin_reward_fail_with_long_label() {
        let mut deps = default_setup();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Coin(CoinReward{
                label: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                coin: Coin { denom: "uaura".to_string(), amount: Uint128::from_str("100").unwrap() },
                number: 100,
                id: 1,
            })
        };

        let res = execute(
            deps.as_mut(), 
            mock_env(), 
            mock_info(CREATOR, &coins(100u128, "uaura".to_string())/* deposit funds */), 
            add_reward
        ).unwrap_err();

        match res {
            ContractError::TextTooLong {} => {}
            _ => panic!()
        }
    }
    
    #[test]
    fn add_coin_reward_fail_with_insufficent_fund() {
        let mut deps = default_setup();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Coin(CoinReward{
                label: "100uaura".to_string(),
                coin: Coin { denom: "uaura".to_string(), amount: Uint128::from_str("100").unwrap() },
                number: 100,
                id: 1,
            })
        };

        let res = execute(
            deps.as_mut(), 
            mock_env(), 
            mock_info(CREATOR, &coins(9999u128, "uaura".to_string())/* insufficent funds, required 10000uaura */), 
            add_reward
        ).unwrap_err();

        match res {
            ContractError::InsufficentFund {} => {}
            _ => panic!()
        }
    }
    // Fungible Token
    #[test]
    fn add_token_reward_success() {
        let mut deps = default_setup();

        let test_address = "cw20";
        let amount = Uint128::from(100u128);
        let number = 100;

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::FungibleToken(TokenReward{
                label: "CW20".to_string(),
                token_address: test_address.to_string(),
                amount,
                number,
                id: 1,
            })
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap();

        let transfer_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: test_address.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom { 
                owner: CREATOR.to_string(), 
                recipient: mock_env().contract.address.to_string(), 
                amount: amount.checked_mul(Uint128::from(number as u128)).unwrap() 
            }).unwrap(),
            funds: vec![],
        });

        assert_eq!(res, Response::new().add_attribute("action", "add_rewards").add_message(transfer_msg));
    }

    #[test]
    fn add_token_reward_fail_with_long_label() {
        let mut deps = default_setup();

        let test_address = "cw20";
        let amount = Uint128::from(100u128);
        let number = 100;

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::FungibleToken(TokenReward{
                label:"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                token_address: test_address.to_string(),
                amount,
                number,
                id: 1,
            })
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap_err();
        match res {
            ContractError::TextTooLong {} => {}
            _ => panic!()
        }
    }
 
    // NFTs collection
    #[test]
    fn add_nfts_reward_success() {
        let mut deps = default_setup();

        let test_address = "cw721";
        let nft_id = "111";

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::NftCollection(CollectionReward{
                label: "BBB collection".to_string(),
                collection_address: test_address.to_string(),
                token_ids: vec![nft_id.to_string()],
                id: 1,
            })
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap();

        let transfer_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: test_address.to_string(),
            msg: to_binary(&CW721ExecuteMsg::<CW721Extension,CW721Extension>::TransferNft {
                recipient: mock_env().contract.address.to_string(), 
                token_id: nft_id.to_string()
            }).unwrap(),
            funds: vec![],
        });

        assert_eq!(res, Response::new().add_attribute("action", "add_rewards").add_message(transfer_msg));
    }   

    #[test]
    fn add_nfts_reward_fail_with_long_label() {
        let mut deps = default_setup();

        let test_address = "cw721";
        let nft_id = "111";

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::NftCollection(CollectionReward{
                label: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                       aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                       aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                       aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                collection_address: test_address.to_string(),
                token_ids: vec![nft_id.to_string()],
                id: 1,
            })
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap_err();
        match res {
            ContractError::TextTooLong {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn add_nfts_reward_fail_with_too_many_nfts() {
        let mut deps = default_setup();

        let test_address = "cw721";

        let mut token_ids: Vec<String> = Vec::with_capacity(65537);
        for _ in 0..token_ids.capacity() {
            token_ids.push("0".to_string());
        }

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::NftCollection(CollectionReward{
                label: "BBB collection".to_string(),
                collection_address: test_address.to_string(),
                token_ids: Vec::from(token_ids),
                id: 1,
            })
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap_err();
        match res {
            ContractError::TooManyNfts {} => {}
            _ => panic!()
        }
    }

    // general fail
    #[test]
    fn add_reward_fail_with_unauthorized() {
        let mut deps = default_setup();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Text(TextReward{
                label: "you lose".to_string(),
                number: 100,
                id: 1,
            }) 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(USER/*sender is user*/, &[]), add_reward).unwrap_err();
        match res {
            ContractError::Unauthorized {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn add_reward_fail_with_wheel_activated() {
        let mut deps = default_setup();

        ADMIN_CONFIG.save(deps.as_mut().storage, &AdminConfig { 
            admin: Addr::unchecked(CREATOR), 
            // set activate to true
            activate: true 
        }).unwrap();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Text(TextReward{
                label: "you lose".to_string(),
                number: 100,
                id: 1,
            }) 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap_err();
        match res {
            ContractError::WheelActivated {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn add_reward_fail_with_too_many_slots() {
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(65536); // rewards reach maximum capacity
        for i in 0..wheel_rewards.capacity() {
            wheel_rewards.push(WheelReward::Text(TextReward{
                label: "you lose".to_string(),
                number: 1,
                id: (i as u32),
            }));
        }

        WHEEL_REWARDS.save(deps.as_mut().storage, &(65536, wheel_rewards)).unwrap();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Text(TextReward{
                label: "you lose".to_string(),
                number: 100,
                id: u32::MAX,
            }) 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap_err();
        match res {
            ContractError::TooManySlots {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn add_reward_fail_with_too_many_rewards() {
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: u32::MAX,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(u32::MAX, wheel_rewards)).unwrap();

        let add_reward = ExecuteMsg::AddReward { 
            reward: WheelReward::Text(TextReward{
                label: "you lose".to_string(),
                number: 1,
                id: 2,
            }) 
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), add_reward).unwrap_err();
        match res {
            ContractError::TooManyRewards {} => {}
            _ => panic!()
        }
    }

    /* ============================================================ RemoveReward ============================================================ */
    #[test]
    fn remove_reward_success() {
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let slot = 0;
        let remove_reward = ExecuteMsg::RemoveReward { 
            slot
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), remove_reward).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "remove_reward")
            .add_attribute("slot", slot.to_string()));

        assert_eq!(WHEEL_REWARDS.load(deps.as_mut().storage).unwrap(), (0, vec![]));
    }

    #[test]
    fn remove_reward_fail_with_unauthorized() {
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(1, wheel_rewards)).unwrap();

        let slot = 0;
        let remove_reward = ExecuteMsg::RemoveReward { 
            slot
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(USER/* sender is user */, &[]), remove_reward).unwrap_err();
        match res {
            ContractError::Unauthorized {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn remove_reward_fail_with_wheel_activated() {
        let mut deps = default_setup();

        ADMIN_CONFIG.save(deps.as_mut().storage, &AdminConfig { 
            admin: Addr::unchecked(CREATOR), 
            // set activate to true
            activate: true 
        }).unwrap();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(1, wheel_rewards)).unwrap();

        let slot = 0;
        let remove_reward = ExecuteMsg::RemoveReward { 
            slot
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), remove_reward).unwrap_err();
        match res {
            ContractError::WheelActivated {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn remove_reward_fail_with_invalid_slot() {
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(1, wheel_rewards)).unwrap();

        let slot = 1;
        let remove_reward = ExecuteMsg::RemoveReward { 
            slot
        };

        let res = execute(deps.as_mut(), mock_env(), mock_info(CREATOR, &[]), remove_reward).unwrap_err();
        match res {
            ContractError::InvalidSlotReward {} => {}
            _ => panic!()
        }
    }

    /* ============================================================ ActivateWheel ============================================================ */
    #[test]
    fn activate_wheel_success() {
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(20000),
            shuffle: Some(true)
        };

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), activate_wheel).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "activate_wheel"));
    }

    #[test]
    fn activate_wheel_fail_with_unauthorized() {
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(20000),
            shuffle: None
        };

        let res = execute(deps.as_mut(), env, mock_info(USER/* sender is user */, &[]), activate_wheel).unwrap_err();
        match res {
            ContractError::Unauthorized {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn activate_wheel_fail_with_wheel_activated() {
        let mut deps = default_setup();

        ADMIN_CONFIG.save(deps.as_mut().storage, &AdminConfig { 
            admin: Addr::unchecked(CREATOR), 
            // set activate to true
            activate: true 
        }).unwrap();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(20000),
            shuffle: None
        };

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), activate_wheel).unwrap_err();
        match res {
            ContractError::WheelActivated {} => {}
            _ => panic!()
        }
    }

    /* ============================================================ WithdrawCoin  ======================================================================== */
    #[test]
    fn withdraw_native_coin_fail_with_insufficent_fund(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(20000),
            shuffle: None
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);
        
        let withdraw_native_coin = ExecuteMsg::WithdrawCoin { 
            recipient: Some("recipient".to_string()), 
            denom: "uaura".to_string()
        };

        let env = env_with_specify(Timestamp::from_seconds(21000), 1);

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), withdraw_native_coin).unwrap_err();
        match res {
            ContractError::InsufficentFund {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn withdraw_native_coin_fail_with_wheel_not_activated(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let withdraw_native_coin = ExecuteMsg::WithdrawCoin { 
            recipient: Some("recipient".to_string()), 
            denom: "uaura".to_string()
        };

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), withdraw_native_coin).unwrap_err();
        match res {
            ContractError::WheelNotActivated {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn withdraw_native_coin_fail_with_unauthorized(){
        let mut deps = default_setup();

        ADMIN_CONFIG.save(deps.as_mut().storage, &AdminConfig { 
            admin: Addr::unchecked(CREATOR), 
            // set activate to true
            activate: true 
        }).unwrap();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let withdraw_native_coin = ExecuteMsg::WithdrawCoin { 
            recipient: Some("recipient".to_string()), 
            denom: "uaura".to_string()
        };

        let res = execute(deps.as_mut(), env, mock_info(USER, &[]), withdraw_native_coin).unwrap_err();
        match res {
            ContractError::Unauthorized {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn withdraw_native_coin_fail_with_wheel_not_end(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(20000),
            shuffle: None
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);
        
        let withdraw_native_coin = ExecuteMsg::WithdrawCoin { 
            recipient: Some("recipient".to_string()), 
            denom: "uaura".to_string()
        };

        let env = env_with_specify(Timestamp::from_seconds(19000) /* < 20000 */, 1);

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), withdraw_native_coin).unwrap_err();
        match res {
            ContractError::WheelNotEnded {} => {}
            _ => panic!()
        }
    }


    /* ============================================================ Spin  ======================================================================== */
    #[test]
    fn spin_fail_with_wheel_not_activated(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let spin_msg = ExecuteMsg::Spin { number: Some(5) };

        let res = execute(deps.as_mut(), env, mock_info(USER, &[]), spin_msg).unwrap_err();
        match res {
            ContractError::WheelNotActivated {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn spin_fail_with_spin_number_larger_than_config(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(40000),
            shuffle: None
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);

        let spin_msg = ExecuteMsg::Spin { number: Some(101) /* max is 10 */ };

        let res = execute(deps.as_mut(), env, mock_info(USER, &[]), spin_msg).unwrap_err();
        match res {
            ContractError::InvalidNumberSpins {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn spin_fail_with_start_time_larger_than_current(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(20000) /* > 15000 */), 
            end_time: Timestamp::from_seconds(40000),
            shuffle: None
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);

        let spin_msg = ExecuteMsg::Spin { number: Some(11) };

        let res = execute(deps.as_mut(), env.clone(), mock_info(USER, &[]), spin_msg).unwrap_err();
        match res {
            ContractError::InvalidNumberSpins {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn spin_fail_with_current_larger_than_end_time(){
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(40000),
            shuffle: None,
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);

        let env = env_with_specify(Timestamp::from_seconds(50000) /* > 40000 */, 1);

        let spin_msg = ExecuteMsg::Spin { number: Some(1) };

        let res = execute(deps.as_mut(), env.clone(), mock_info(USER, &[]), spin_msg).unwrap_err();
        match res {
            ContractError::WheelEnded {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn spin_fail_with_spin_amount_not_enough(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(40000),
            shuffle: None,
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);

        let spin_msg = ExecuteMsg::Spin { number: Some(1) };

        let res = execute(
            deps.as_mut(), 
            env.clone(), 
            mock_info(USER, &[Coin { denom: "uaura".to_string(), amount: Uint128::from(999u128/* Not enough fund */) }]), 
            spin_msg).unwrap_err();
        match res {
            ContractError::InsufficentFund {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn spin_success(){
        let mut deps = default_setup();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));
        
        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(40000),
            shuffle: None,
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);

        let spin_msg = ExecuteMsg::Spin { number: Some(1) };

        let res = execute(
            deps.as_mut(), 
            env.clone(), 
            mock_info(USER, &[Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }]), 
            spin_msg).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "spin")
        .add_attribute("sender", USER)
        .add_attribute("spins", "1"));
    }

    /* ============================================================ Withdraw  ======================================================================== */
    #[test]
    fn withdraw_reward_success(){
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(20000),
            shuffle: None
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);
        
        let withdraw_reward = ExecuteMsg::Withdraw { 
            recipient: Some("recipient".to_string()), 
            slot: 0
        };

        let env = env_with_specify(Timestamp::from_seconds(21000), 1);

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), withdraw_reward).unwrap();
        assert_eq!(res, Response::new().add_attribute("action", "withdraw").add_attribute("slot", "0"))
        
    }

    #[test]
    fn withdraw_reward_fail_with_wheel_not_activated(){
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let withdraw_reward = ExecuteMsg::Withdraw { 
            recipient: Some("recipient".to_string()), 
            slot: 0
        };

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), withdraw_reward).unwrap_err();
        match res {
            ContractError::WheelNotActivated {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn withdraw_reward_fail_with_unauthorized(){
        let mut deps = default_setup();
        
        ADMIN_CONFIG.save(deps.as_mut().storage, &AdminConfig { 
            admin: Addr::unchecked(CREATOR), 
            // set activate to true
            activate: true 
        }).unwrap();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let withdraw_reward = ExecuteMsg::Withdraw { 
            recipient: Some("recipient".to_string()), 
            slot: 0
        };

        let res = execute(deps.as_mut(), env, mock_info(USER, &[]), withdraw_reward).unwrap_err();
        match res {
            ContractError::Unauthorized {} => {}
            _ => panic!()
        }
    }

    #[test]
    fn withdraw_reward_fail_with_wheel_not_end(){
        let mut deps = default_setup();

        let mut wheel_rewards: Vec<WheelReward> = Vec::with_capacity(1);
        // add reward
        wheel_rewards.push(WheelReward::Text(TextReward{
            label: "you lose".to_string(),
            number: 100,
            id: 1,
        }));

        WHEEL_REWARDS.save(deps.as_mut().storage, &(100, wheel_rewards)).unwrap();

        let env = env_with_specify(Timestamp::from_seconds(15000), 1);

        let activate_wheel = ExecuteMsg::ActivateWheel { 
            price: Coin { denom: "uaura".to_string(), amount: Uint128::from(1000u128) }, 
            start_time: Some(Timestamp::from_seconds(10000)), 
            end_time: Timestamp::from_seconds(20000),
            shuffle: None
        };

        _ = execute(deps.as_mut(), env.clone(), mock_info(CREATOR, &[]), activate_wheel);
        
        let withdraw_reward = ExecuteMsg::Withdraw { 
            recipient: Some("recipient".to_string()), 
            slot: 0
        };

        let env = env_with_specify(Timestamp::from_seconds(19000) /* < 20000 */, 1);

        let res = execute(deps.as_mut(), env, mock_info(CREATOR, &[]), withdraw_reward).unwrap_err();
        match res {
            ContractError::WheelNotEnded {} => {}
            _ => panic!()
        }
    }
}

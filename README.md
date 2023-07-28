# WHEEL-OF-FORTUNE

## EXECUTE METHODS

### ACTIVATE-WHEEL

 Activate wheel. After activated, wheel can not be modified
```rust
ActivateWheel {
    fee: UserFee, // fee pay for spin
    start_time: Option<Timestamp>, // start time of wheel, default is current block time
    end_time: Timestamp, // end time of wheel
    shuffle: Option<bool>, // shuffle wheel rewards if true, default `false`
}
```
- Only allow `Admin` to execute
- Can only be executed when **Wheel** is not activated

###  ADD-WHITE-LIST

 Add wallet addresses considered to be acceptable to spin in wheel if it's private

```rust
AddWhitelist {
    addresses: Vec<String> // wallet addresses
}

/* Example:
    add_whitelist {
        addresses: ["aura159mt7ryhxd9g07fjw5lpreqnv8yzuf72vh22zg","aura18xyvzh6xha7c0wmj0unaav5yhgjy25j0uy933k"]
    }
*/
```
- Only allow `Admin` to execute
- Can only be executed when **Wheel** is not activated

### REMOVE-WHITE-LIST
 Remove wallet addressese from white list
```rust
RemoveWhitelist {
    addresses: Vec<String> // wallet addresses
}

/* Example:
    remove_whitelist {
        addresses: ["aura159mt7ryhxd9g07fjw5lpreqnv8yzuf72vh22zg","aura18xyvzh6xha7c0wmj0unaav5yhgjy25j0uy933k"]
    }
*/
```
- Only allow `Admin` to execute
- Can only be executed when **Wheel** is not activated

### ADD-REWARD
 Add reward to wheel, slot by slot
```rust
// NFTs collection 
#[cw_serde]
pub struct CollectionReward {
    pub label: String, // label of slot
    pub collection_address: String, // nft contract address 
    pub token_ids: Vec<String> // list of token id, it's length is number of nft items in slot
}

#[cw_serde]
pub struct TokenReward {
    pub label: String, // label of slot
    pub token_address: String, // token contract address
    pub amount: Uint128, // token amount
    pub number: u32 // number of token items in slot
}

// Token
#[cw_serde]
pub struct CoinReward {
    pub label: String, // label of slot
    pub coin: Coin, // coin amount etc 100uaura
    pub number: u32 // number of coin items in slot
}

// Text
#[cw_serde]
pub struct TextReward {
    pub label: String, // label of slot
    pub number: u32, // number of text items in slot
}

// wheel reward can be `nft`, `token`, `coin` or `text`
#[cw_serde]
pub enum WheelReward {
    NftCollection(CollectionReward),
    FungibleToken(TokenReward),
    Coin(CoinReward),
    Text(TextReward)
}

AddReward {
    reward: WheelReward // wheel reward
}

/* Example
    add_reward {
        reward: {
            nft_collection{
                label: "BBB collection",
                collection_address: "aura1gud6mupw5cg255yk84xc4xd0dcxggpa48m58vrakam96xgaz6xvq7kwsmf",
                token_ids: ["111","222","333","666"] 
            }
        }
    }

    add_reward {
        reward: {
            token {
                label: "CW20 Fungible Token",
                token_address: "aura1gud6mupw5cg255yk84xc4xd0dcxggpa48m58vrakam96xgaz6xvq7kwsmf",
                amount: "1000"
                number: 100 
            }
        }
    }

    add_reward {
        reward: {
            coin {
                label: "Aura token",
                coin: "300uaura",
                number: 100 
            }
        }
    }

    add_reward {
        reward: {
            text {
                label: "you lose",
                number: 100 
            }
        }
    }
*/
```
- Only allow `Admin` to execute
- Can only be executed when **Wheel** is not activated

### REMOVE-REWARD
 Remove wheel reward at speicfic slot
```rust
RemoveReward {
    slot: usize // slot 
}

/* Example
    remove_reward {
        slot: 3
    }
*/
```
- Only allow `Admin` to execute
- Can only be executed when **Wheel** is not activated

### WITHDRAW
 Withdraw reward from contract
```rust
Withdraw {
    recipient: Option<String>, // recipient of coin, default is contract owner
    slot: u32, // slot reward
}

/* Example:
    withdraw {
        recipient: "aura159mt7ryhxd9g07fjw5lpreqnv8yzuf72vh22zg",
        slot: 0
    }
*/
```
- Only allow `Admin` to execute
- Can only be executed when **Wheel** is activated and ended

### WITHDRAW-COIN
 Withdraw coins from contract
```rust
WithdrawCoin {
    denom: String, // denom
    recipient: Option<String> // recipient of coin, default is contract owner
}
```
- Only allow `Admin` to execute

### SPIN
 User spin wheel for reward and fun
```rust
Spin {
    number: Option<u32> // number of turns, default is 1
},

/* Example:
    spin {
        number: 5
    }
*/
```
- Anyone can execute in `public` mode
- Only whitelist can execute in `private` mode
- Players have to pay for each spin
- Can only be executed whe **wheel** is activated and operation

### CLAIM-REWARD
 Player claim rewards
```rust
ClaimReward {
    rewards: Vec<usize> // indexes of reward that want to claim
},

/* Example:
    claim_reward {
        rewards: [1,2,3,4]
    }
*/
``` 
- Players can only claim the rewards they have won
- Can only be executed whe **wheel** is activated

### NOIS-RECEIVE
 Method that reveive callback from `nois-proxy` contract
```rust
#[cw_serde]
pub struct NoisCallback {
    /// The ID chosen by the caller for this job. Use this field to map responses to requests.
    pub job_id: String,
    /// The point in time when the randomness was first published. This information is provided
    /// by the randomness provider. This is not the time when the randomness was processed on chain.
    pub published: Timestamp,
    /// The randomness. This is guaranteed to be 32 bytes long.
    pub randomness: HexBinary,
}

NoisReceive {
    callback: NoisCallback // callback params
}

/* Example:
    nois_receive {
        callback: {
            job_id: "job id test",
            published: "1686815501000000",
            randomness: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }
    }
*/
```
- Can only be executed by `nois-proxy` contract

## QUERY METHODS

### GET-WHEEL-REWARDS
 Get information of all slot rewards in wheel
```rust
GetWheelRewards{}
```

### GET-PLAYER-REWARDS
 Get information of all rewards that player have won
```rust
GetPlayerRewards{
    address: String // wallet address of player
}
```

### GET-PLAYER-SPINNED
 Get the number of turns spinned by the player
```rust
GetPlayerSpinned{
    address: String // wallet address of player
}
```

### GET-WHITELIST
 Get a list of addresses that can spin the wheel in *private* mode
```rust
GetWhiteList{}
```

### GET-WHEEL-CONFIG
 Get config of wheel
```rust
GetWheelConfig{}
```

### SPINNABLE
 Check if the address can spin the wheel and get the number of turns left
 ```rust
 Spinnable{
    address: String // wallet address of player
 }
 ```

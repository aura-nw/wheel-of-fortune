# WHEEL-OF-FORTUNE

## I. Run a LocalNet

</br>

**Prerequisite**
- Go 1.18
- Ignite v0.22.1

</br>

```
git clone https://github.com/aura-nw/aura.git
cd aura
ignite chain serve -v
```

</br>

## II. Deploy contracts

</br>

**CW20**
```javascript
export CW20_WASM_PATH="/wheel-of-fortune/artifacts/cw20_token.wasm"
export CANTHO_ADDR=$(aurad keys show Cantho -a)

aurad tx wasm store \
    $CW20_WASM_PATH \
    --from Cantho \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.4  \
    --gas-prices 0.025uaura \
    --broadcast-mode=block

aurad tx wasm instantiate \
	1 '{"name":"cw20 reward","symbol":"test","decimals":10,"initial_balances":[{"address":"'$CANTHO_ADDR'","amount":"10000"}]}' \
	--label "cw20 reward" \
	--no-admin \
	--from Cantho \
	--chain-id aura-testnet \
        --gas=auto \
        --gas-adjustment 1.4  \
        --gas-prices 0.025uaura \
        --broadcast-mode=block

// store contract address for bellow commands
```
</br>

**CW721**
```javascript
export CW721_WASM_PATH="/wheel-of-fortune/artifacts/cw721_reward.wasm"
export CANTHO_ADDR=$(aurad keys show Cantho -a)

aurad tx wasm store \
       $CW721_WASM_PATH \
       --from Cantho \
       --chain-id aura-testnet \
       --gas=auto \
       --gas-adjustment 1.4  \
       --gas-prices 0.025uaura \
       --broadcast-mode=block

aurad tx wasm instantiate \
	2 '{"name":"cw721 reward","symbol":"test","minter":"'$CANTHO_ADDR'"}' \
	--label "cw721 reward" \
	--no-admin \
	--from Cantho \
	--chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.4  \
    --gas-prices 0.025uaura \
    --broadcast-mode=block

// store contract address for bellow commands
```

</br>

**WHEEL**
```javascript
export WHEEL_WASM_PATH="/wheel-of-fortune/artifacts/wheel_of_fortune.wasm"
export CANTHO_ADDR=$(aurad keys show Cantho -a)

aurad tx wasm store \
       $WHEEL_WASM_PATH \
       --from Cantho \
       --chain-id aura-testnet \
       --gas=auto \
       --gas-adjustment 1.4  \
       --gas-prices 0.025uaura \
       --broadcast-mode=block

aurad tx wasm instantiate \
	3 '{"wheel_name":"test","random_seed":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","max_spins_per_address":5,"is_public":true,"is_advanced_randomness":false,"nois_proxy":"aura159mt7ryhxd9g07fjw5lpreqnv8yzuf72vh22zg"}' \
	--label "wheel of fortune" \
	--no-admin \
	--from Cantho \
	--chain-id aura-testnet \
        --gas=auto \
        --gas-adjustment 1.4  \
        --gas-prices 0.025uaura \
        --broadcast-mode=block

// store contract address for bellow commands
```

## III. Add Rewards

</br>

**Nft Collection**
```javascript
export TOKEN_ID=1
export CANTHO_ADDR=$(aurad keys show Cantho -a)
export CW721_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>

// mint nft
aurad tx wasm execute aura1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqz9up5g \
    '{"mint":{"token_id":"'$TOKEN_ID'","owner":"'$CANTHO_ADDR'","extension":{}}}' \
    --from Cantho \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3

// approve
aurad tx wasm execute $CW721_CONTRACT_ADDR \
    '{"approve_all":{"operator":"'$WHEEL_CONTRACT_ADDR'"}}' \
    --from Cantho \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3

// add reward
aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
    '{"add_reward":{"reward": {"nft_collection":{"label":"BBB collection","collection_address":"'$CW721_CONTRACT_ADDR'","token_ids":["'$TOKEN_ID'"]}}}}' \
    --from Cantho \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3
```

</br>

**CW20**
```Javascript
export CW20_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export AMOUNT=10000

// increase allowance
aurad tx wasm execute $CW20_CONTRACT_ADDR \
    '{"increase_allowance":{"spender":"'$WHEEL_CONTRACT_ADDR'","amount":"'$AMOUNT'"}}' \
    --from Cantho \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3

// add reward
aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
    '{"add_reward":{"reward": {"fungible_token":{"label":"CW20","token_address":"'$CW20_CONTRACT_ADDR'","amount":"100","number":100}}}}' \
    --from Cantho \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3
```

</br>

**Coin**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
    '{"add_reward":{"reward": {"coin":{"label":"200uaura","coin":{"denom":"uaura","amount":"200"},"number":50}}}}' \
    --from Cantho \
	--amount 10000uaura \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3
```

</br>

**Text**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
    '{"add_reward":{"reward": {"text":{"label":"you lose","number":200}}}}' \
    --from Cantho \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3
```

</br>

**Get All Rewards**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>

aurad query wasm contract-state smart \
    $WHEEL_CONTRACT_ADDR \
    '{"get_wheel_rewards": {}}'
```

## IV. Activate and Spin

</br>

**Activate**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>
// UTC+0 timestamp in nano seconds
export END_TIME=<<TIMESTAMP>>

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
    '{"activate_wheel":{"end_time":"'$END_TIME'","fee":{"denom":"uaura","spin_price":"1000","nois_fee":"300"}}}' \
    --from Cantho \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3
```

</br>

**Spin**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
    '{"spin":{"number":5}}' \
    --from Vinh \
	--amount 5000uaura \
    --gas-prices 0.025uaura \
    --chain-id aura-testnet \
    --gas=auto \
    --gas-adjustment 1.3
```

## V. Player Get Rewards and Claim

</br>

**Get Rewards**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export VINH_ADDR=$(aurad keys show Vinh -a)

aurad query wasm contract-state smart \
        $WHEEL_CONTRACT_ADDR \
        '{"get_player_rewards": {"address":"'$VINH_ADDR'"}}'
```

</br>

**Claim**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export VINH_ADDR=$(aurad keys show Vinh -a)

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
        '{"claim_reward":{"rewards":[0,1,2,3,4]}}' \
        --from Vinh \
        --gas-prices 0.025uaura \
        --chain-id aura-testnet \
        --gas=auto \
        --gas-adjustment 1.3
```

## VI. Onwer withdraw

</br>

**Withdraw Coin**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export DENOM="uaura"

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
        '{"withdraw":{"denom":"'$DENOM'"}}' \
        --from Cantho \
        --gas-prices 0.025uaura \
        --chain-id aura-testnet \
        --gas=auto \
        --gas-adjustment 1.3
```

</br>

**Withdraw CW20 Token**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export CW20_CONTRACT_ADDR=<<CONTRACT_ADDT>>

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
        '{"withdraw_token":{"token_address":"'$CW20_CONTRACT_ADDR'"}}' \
        --from Cantho \
        --gas-prices 0.025uaura \
        --chain-id aura-testnet \
        --gas=auto \
        --gas-adjustment 1.3
```

</br>

**Withdraw CW721**
```Javascript
export WHEEL_CONTRACT_ADDR=<<CONTRACT_ADDR>>
export CW721_CONTRACT_ADDR=<<CONTRACT_ADDT>>
// example remaining token id
export TOKEN_IDS=[1,2,3,4,5]

aurad tx wasm execute $WHEEL_CONTRACT_ADDR \
        '{"withdraw_nft":{"collection":"'$CW721_CONTRACT_ADDR'", "token_ids":'$TOKEN_IDS'}}' \
        --from Cantho \
        --gas-prices 0.025uaura \
        --chain-id aura-testnet \
        --gas=auto \
        --gas-adjustment 1.3
```


# Tic Tac Toe Game (Cosmwasm)

## PATH Variables
- export RPC=https://rpc.malaga-420.cosmwasm.com:443
- export NODE=(--node $RPC)
- export CHAIN_ID=malaga-420
- export FEE_DENOM=umlg
- export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 0.25$FEE_DENOM --gas auto --gas-adjustment 1.3)

## Deploy
- cargo unit-test
- RUSTFLAGS='-C link-arg=-s' cargo wasm
- RES=$(wasmd tx wasm store target/wasm32-unknown-unknown/release/cw_tictactoe.wasm --from wallet $TXFLAG -y --output json -b block)
- CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
- INIT='{}'
- wasmd tx wasm instantiate $CODE_ID "$INIT" --from wallet --label "tictactoe" $TXFLAG -y --no-admin
- CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
- wasmd query wasm contract-state all $CONTRACT $NODE

## Deployed Contract (https://rpc.malaga-420.cosmwasm.com:443)
- CONTRACT=wasm1rcxgwfkxfvw7279lzwp4pe30z9pukkr3u7gpx783aerjva6sjxcqyj3lkv

## Invite users
- NEW_INVITE='{"invite":{"name":"Alice"}}'
- wasmd tx wasm execute $CONTRACT "$NEW_INVITE" --amount 100umlg --from wallet $TXFLAG -y

## Accept invites
- INVITE_ACCEPTED='{"accepted":{"name":"Bob", "game_id":"1"}}'
- wasmd tx wasm execute $CONTRACT "$INVITE_ACCEPTED" --amount 100umlg --from wallet2 $TXFLAG -y

## Play game
- PLAY_GAME='{"play":{"position_x":0, "position_y":0}}'
- wasmd tx wasm execute $CONTRACT "$PLAY_GAME" --amount 100umlg --from wallet $TXFLAG -y

## Game status query
- GAME_STATUS_QUERY='{"game_status":{"game_id":"1"}}'
- wasmd query wasm contract-state smart $CONTRACT "$GAME_STATUS_QUERY" $NODE --output json

## Board status query
- BOARD_STATUS_QUERY='{"board_status":{"game_id":"1"}}'
- wasmd query wasm contract-state smart $CONTRACT "$BOARD_STATUS_QUERY" $NODE --output json

## Open invites query (list of all open game ids)
- OPEN_INVITES_QUERY='{"open_invites": {}}'
- wasmd query wasm contract-state smart $CONTRACT "$OPEN_INVITES_QUERY" $NODE --output json

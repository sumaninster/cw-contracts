# Tic Tac Toe Game

## Invite users
NEW_INVITE='{"invite":{"name":"Alice"}}'
wasmd tx wasm execute $CONTRACT "NEW_INVITE" --amount 100umlg --from wallet $TXFLAG -y

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
- wasmd query wasm contract-state smart $CONTRACT "BOARD_STATUS_QUERY" $NODE --output json

## Open invites query (list of all open game ids)
- OPEN_INVITES_QUERY='{"open_invites": {}}'
- wasmd query wasm contract-state smart $CONTRACT "$OPEN_INVITES_QUERY" $NODE --output json

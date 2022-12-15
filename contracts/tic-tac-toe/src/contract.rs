use cosmwasm_std::{entry_point, Binary, DepsMut, Env, MessageInfo, Response, StdError, Deps, StdResult, to_binary};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use crate::error::ContractError;
use crate::msg::{BoardStatusResponse, ExecuteMsg, GameStatusResponse, InstantiateMsg, OpenInvitesResponse, QueryMsg};
use crate::state::{GAME_SESSION, GAME_INVITES, GAME_ID_INC, GameId, GameInvite, GameSession, Game, Player, PlayerDetails, USER_SESSION, OPEN_INVITES, OpenInvites, FIRST_PLAYER};

const MIN_NAME_LENGTH: u64 = 3;
const MAX_NAME_LENGTH: u64 = 64;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    GAME_ID_INC.save(deps.storage, &0)?;
    let invites = vec![];
    OPEN_INVITES.save(deps.storage, &OpenInvites{ invites })?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Invite { name }=> execute_invite(deps, env, info, name),
        ExecuteMsg::Accepted { name, game_id } => execute_accepted(deps, env, info, name, game_id),
        ExecuteMsg::Play { position_x, position_y } => execute_play(deps, env, info, position_x, position_y),
    }
}
///Process transaction for new game invites
pub fn execute_invite(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    validate_name(&name)?;
    let mut game_id = GAME_ID_INC.load(deps.storage)?;
    game_id += 1;
    GAME_ID_INC.save(deps.storage, &game_id)?;
    let new_invite = GameInvite { invite_player: PlayerDetails{
        addr: info.sender.clone(),
        name,
        player: None
    }, accepted_player: None,
        accepted: false,
    };
    GAME_INVITES.save(deps.storage, game_id, &new_invite)?;
    USER_SESSION.save(deps.storage, info.sender, &game_id)?;

    let mut invites = OPEN_INVITES.load(deps.storage)?.invites;
    invites.push(game_id);
    OPEN_INVITES.save(deps.storage, &OpenInvites{ invites })?;
    Ok(Response::default())
}
///Process transaction for accepted game invites
pub fn execute_accepted(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    game_id: GameId,
) -> Result<Response, ContractError> {
    validate_name(&name)?;
    let mut invite = GAME_INVITES.update(deps.storage, game_id, |record| {
        if let Some(mut record) = record {
            if info.sender == record.invite_player.addr {
                return Err(ContractError::YouCantPlayWithYourSelf {});
            }
            record.accepted_player = Some(PlayerDetails{
                addr: info.sender.clone(),
                name,
                player: None
            });
            record.accepted = true;
            Ok(record)
        } else {
            Err(ContractError::GameIdNotFound { game_id })
        }
    })?;
    if invite.accepted {
        let first_player = get_first_player(&mut invite)?;
        let new_game = GameSession {
            game: Game::new(first_player),
            players: invite.clone(),
            winner: None,
            game_over: false
        };
        GAME_SESSION.save(deps.storage, game_id, &new_game)?;
        USER_SESSION.save(deps.storage, info.sender, &game_id)?;
        GAME_INVITES.save(deps.storage, game_id, &invite)?;

        let mut invites = OPEN_INVITES.load(deps.storage)?.invites;
        let index = invites.binary_search(&game_id).expect("Game Id not found");
        invites.remove(index);
        OPEN_INVITES.save(deps.storage, &OpenInvites{ invites })?;
    }
    Ok(Response::default())
}
///Process transaction for game moves from each player
pub fn execute_play(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    position_x: u8,
    position_y: u8,
) -> Result<Response, ContractError> {
    let game_id = match USER_SESSION.may_load(deps.storage, info.sender.clone())? {
        Some(g) => g.clone(),
        None => return Err(ContractError::YourSessionNotFound {})
    };
    let winner = GAME_SESSION.update(deps.storage, game_id, |record| -> Result<GameSession, ContractError> {
        if let Some(mut record) = record {
            if record.game_over {
                let winner = match record.winner {
                    Some(w) => w.name,
                    None => "None, Draw".to_string(),
                };
                return Err(ContractError::GameOver { game_id, status: "Game Over".to_string(), winner });
            } else if info.sender != record.game.current_player.addr {
                return Err(ContractError::ItsNotYourTurn { game_id });
            } else if !record.players.accepted {
                return Err(ContractError::InviteNotAcceptedYet { game_id });
            } else if record.game.is_valid_move((position_x, position_y)) {
                record.game.board[position_x as usize][position_y as usize] = Some(record.game.current_player.player.unwrap());
                record.game.next_player(&record.players);
                let (winner, over) = record.game.is_game_over(&record.players);
                if over {
                    record.winner = winner.clone();
                    record.game_over = true;
                }
            } else {
                return Err(ContractError::InvalidMove { game_id })
            }
            Ok(record)
        } else {
            return Err(ContractError::GameIdNotFound { game_id })
        }
    })?;
    let mut res = Response::default();
    if winner.game_over {
        let winner = match winner.winner {
            Some(w) => w.name,
            None => "None, Draw".to_string(),
        };
        res = res.add_attribute("status", "Game Over");
        res = res.add_attribute("winner", winner);
    }
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GameStatus { game_id } => query_game_status(deps, env, game_id),
        QueryMsg::BoardStatus { game_id } => query_board_status(deps, env, game_id),
        QueryMsg::OpenInvites {} => query_open_invites(deps, env),
    }
}
///Query for game status: Is game over or in progress, winner name or draw
fn query_game_status(deps: Deps, _env: Env, game_id: GameId) -> StdResult<Binary> {
    let (status, winner) = match GAME_SESSION.may_load(deps.storage, game_id)? {
        Some(record) => {
            if record.game_over {
                let status = "Game Over".to_string();
                let winner = match record.winner {
                    Some(w) => w.name,
                    None => "None, Draw".to_string(),
                };
                (status, winner)
            } else {
                ("Game in progress".to_string(), "None".to_string())
            }
        },
        None => ("Game not found".to_string(), "None".to_string()),
    };
    let resp = GameStatusResponse { game_id, status, winner };
    to_binary(&resp)
}
///Query for board status: get all the cell positions
fn query_board_status(deps: Deps, _env: Env, game_id: GameId) -> StdResult<Binary> {
    let board = GAME_SESSION.load(deps.storage, game_id)?.game.board;
    let resp = BoardStatusResponse { board };
    to_binary(&resp)
}
///Query for open invites: get list of all open game ids
fn query_open_invites(deps: Deps, _env: Env) -> StdResult<Binary> {
    let invites = OPEN_INVITES.load(deps.storage)?.invites;
    let resp = OpenInvitesResponse { invites };
    to_binary(&resp)
}
///Find the first player: The roles of “X” and “O” are decided as follows. The user's public keys are concatenated and the result is hashed. If the first bit of the output is 0, then the game's initiator (whoever posted the invitation) plays "O" and the second player plays "X" and vice versa. “X” has the first move.
fn get_first_player(game_invite: &mut GameInvite) -> Result<PlayerDetails, StdError> {
    let keys = game_invite.invite_player.addr.to_string() + game_invite.accepted_player.as_ref().unwrap().addr.as_str();
    let hash = calculate_hash(keys);
    let mut first_player;
    if is_first_bit_zero(hash) {
        first_player = game_invite.accepted_player.clone().unwrap();
    } else {
        first_player = game_invite.invite_player.clone();
    }
    first_player.player = Some(FIRST_PLAYER);
    if first_player.addr == game_invite.invite_player.addr {
        game_invite.invite_player.player = Some(Player::X);
        game_invite.accepted_player.as_mut().map(|mut x| x.player = Some(Player::O));
    } else {
        game_invite.invite_player.player = Some(Player::O);
        game_invite.accepted_player.as_mut().map(|mut x| x.player = Some(Player::X));
    }
    Ok(first_player)
}
///Calculate has of a string
fn calculate_hash(t: String) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
///Check if first bit is zero
fn is_first_bit_zero(num: u64) -> bool {
    let res = num & (1<<0);
    res.eq(&0)
}
///Check if the char in name is invalid (space is include for names with surname)
fn invalid_char(c: char) -> bool {
    let is_valid =
        c.is_ascii_digit() || c.is_ascii_lowercase() || c.is_ascii_uppercase() || (c == '.' || c == '-' || c == '_' || c == ' ');
    !is_valid
}
/// validate_name returns an error if the name is invalid
/// (we require 3-64 lowercase or uppercase ascii letters, numbers, apace or . - _)
fn validate_name(name: &str) -> Result<(), ContractError> {
    let length = name.len() as u64;
    if (name.len() as u64) < MIN_NAME_LENGTH {
        Err(ContractError::NameTooShort {
            length,
            min_length: MIN_NAME_LENGTH,
        })
    } else if (name.len() as u64) > MAX_NAME_LENGTH {
        Err(ContractError::NameTooLong {
            length,
            max_length: MAX_NAME_LENGTH,
        })
    } else {
        match name.find(invalid_char) {
            None => Ok(()),
            Some(bytepos_invalid_char_start) => {
                let c = name[bytepos_invalid_char_start..].chars().next().unwrap();
                Err(ContractError::InvalidCharacter { c })
            }
        }
    }
}
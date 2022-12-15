use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};

#[cw_serde]
#[derive(Copy, Eq)]
pub enum Player {
    X,
    O,
}

#[cw_serde]
pub struct PlayerDetails {
    pub addr: Addr,
    pub name: String,
    pub player: Option<Player>,
}

#[cw_serde]
pub struct Game {
    pub board: Vec<Vec<Option<Player>>>,
    pub current_player: PlayerDetails,
}

#[cw_serde]
pub struct GameInvite {
    pub invite_player: PlayerDetails,
    pub accepted_player: Option<PlayerDetails>,
    pub accepted: bool,
}

#[cw_serde]
pub struct GameSession {
    pub game: Game,
    pub players: GameInvite,
    pub winner: Option<PlayerDetails>,
    pub game_over: bool,
}

#[cw_serde]
pub struct OpenInvites {
    pub invites: Vec<GameId>,
}

pub type GameId = u128;

pub const GRID_SIZE:usize = 3;
pub const WIN_COND:usize = 3;
pub const FIRST_PLAYER:Player = Player::X;
pub const GAME_ID_INC: Item<GameId> = Item::new("game_id");
pub const GAME_INVITES: Map<GameId, GameInvite> = Map::new("game_invites");
pub const OPEN_INVITES: Item<OpenInvites> = Item::new("open_invites");
pub const GAME_SESSION: Map<GameId, GameSession> = Map::new("game_session");
pub const USER_SESSION: Map<Addr, GameId> = Map::new("user_session");

impl Game {
    pub fn new(first_player: PlayerDetails) -> Game {
        Game {
            board: vec![vec![None;GRID_SIZE];GRID_SIZE],
            current_player: first_player
        }
    }

    pub fn next_player(&mut self, players: &GameInvite) {
        if self.current_player.player == players.invite_player.player {
            self.current_player = players.accepted_player.clone().unwrap()
        } else {
            self.current_player = players.invite_player.clone()
        }
    }

    pub fn is_game_over(&self, players: &GameInvite) -> (Option<PlayerDetails>, bool) {
        match self.get_winner() {
            Some(winner) => {
                if winner == players.invite_player.player.unwrap() {
                    (Some(players.invite_player.clone()), true)
                } else {
                    (Some(players.accepted_player.clone().unwrap()), true)
                }
            },
            None => {
                if self.is_ended() {
                    (None, true)
                } else {
                    (None, false)
                }
            }
        }
    }

    pub fn is_ended(&self) -> bool {
        self.board.iter().all(|row| {
            row.iter().all(|cell| cell.is_some())
        })
    }

    pub fn get_winner(&self) -> Option<Player> {
        macro_rules! has {
            ($player:expr, $x:expr, $y:expr) => {
                self.board[$x][$y] == Some(*$player)
            };
        }
        for player in &[Player::X, Player::O] {
            for i in 0..GRID_SIZE {
                let mut rw = 0;
                let mut cw = 0;
                for j in 0..GRID_SIZE {
                    if has!(player, i, j) {
                        rw += 1;
                        if rw >= WIN_COND {
                            return Some((*player).clone());
                        }
                    } else {
                        rw = 0;
                    }
                    if has!(player, j, i) {
                        cw += 1;
                        if cw >= WIN_COND {
                            return Some((*player).clone());
                        }
                    } else {
                        cw = 0;
                    }
                }
            }
            let mut d1 = 0;
            let mut d2 = 0;
            let l = GRID_SIZE - 1;
            for i in 0..GRID_SIZE {
                if has!(player, i, i) {
                    d1 += 1;
                    if d1 >= WIN_COND {
                        return Some((*player).clone());
                    }
                } else {
                    d1 = 0;
                }
                if has!(player, i, l-i) {
                    d2 += 1;
                    if d2 >= WIN_COND {
                        return Some((*player).clone());
                    }
                } else {
                    d2 = 0;
                }
            }
        }
        None
    }

    pub fn is_valid_move(&self, position: (u8, u8)) -> bool {
        if position.0 as usize >= GRID_SIZE || position.1 as usize >= GRID_SIZE {
            return false;
        }
        self.board[position.0 as usize][position.1 as usize].is_none()
    }
}
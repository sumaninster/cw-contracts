use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};

#[cw_serde]
#[derive(Copy, Eq)]
///Roles for player
pub enum Player {
    X,
    O,
}

#[cw_serde]
///Player details that stores address, name and roles
pub struct PlayerDetails {
    pub addr: Addr,
    pub name: String,
    pub player: Option<Player>,
}

#[cw_serde]
///Stores board state and current player information
pub struct Game {
    pub board: Vec<Vec<Option<Player>>>,
    pub current_player: PlayerDetails,
}

#[cw_serde]
///Players information: Invitee player, player who accepted and accepted status
pub struct GameInvite {
    pub invite_player: PlayerDetails,
    pub accepted_player: Option<PlayerDetails>,
}

#[cw_serde]
///Stores game status: current situation of the game
pub enum GameStatus {
    Playing,
    Draw,
    Winner(PlayerDetails)
}

#[cw_serde]
///Stores game session: game status, players information, final winner (None, if no winner) and game over status
pub struct GameSession {
    pub game: Game,
    pub players: GameInvite,
    pub status: GameStatus,
}

#[cw_serde]
///List of open invites (game ids)
pub struct OpenInvites {
    pub invites: Vec<GameId>,
}
///Game id type
pub type GameId = u128;
///Grid size (3 by 3)
pub const GRID_SIZE:usize = 3;
///Win condition: all 3 rows or cols or diagonal needs to be same
pub const WIN_COND:usize = 3;
///First player role
pub const FIRST_PLAYER:Player = Player::X;
///Stores incremented game id
pub const GAME_ID_INC: Item<GameId> = Item::new("game_id");
///Stores player information and new invites
pub const GAME_INVITES: Map<GameId, GameInvite> = Map::new("game_invites");
///Stores list of open game ids
pub const OPEN_INVITES: Item<OpenInvites> = Item::new("open_invites");
///Stores game session information
pub const GAME_SESSION: Map<GameId, GameSession> = Map::new("game_session");
///Stores game id mapped to user address
pub const USER_SESSION: Map<Addr, GameId> = Map::new("user_session");

impl Game {
    ///Initiate new game, initialize the board and current player with first player
    pub fn new(first_player: PlayerDetails) -> Game {
        Game {
            board: vec![vec![None;GRID_SIZE];GRID_SIZE],
            current_player: first_player
        }
    }
    ///Finds the next player, alternate between two players
    pub fn next_player(&mut self, players: &GameInvite) {
        if self.current_player.player == players.invite_player.player {
            self.current_player = players.accepted_player.clone().unwrap()
        } else {
            self.current_player = players.invite_player.clone()
        }
    }
    ///Checks if the game is over and returns winner (None, if there is no winner)
    pub fn is_game_over(&self, players: &GameInvite) -> GameStatus {
        match self.get_winner() {
            Some(winner) => {
                if winner == players.invite_player.player.unwrap() {
                    GameStatus::Winner(players.invite_player.clone())
                } else {
                    GameStatus::Winner(players.accepted_player.clone().unwrap())
                }
            },
            None => {
                if self.is_ended() {
                    GameStatus::Draw
                } else {
                    GameStatus::Playing
                }
            }
        }
    }
    ///Checks is all the cells are full (or used)
    pub fn is_ended(&self) -> bool {
        self.board.iter().all(|row| {
            row.iter().all(|cell| cell.is_some())
        })
    }
    ///Checks all valid win conditions
    pub fn get_winner(&self) -> Option<Player> {
        macro_rules! has {
            ($player:expr, $x:expr, $y:expr) => {
                self.board[$x][$y] == Some(*$player)
            };
        }
        for player in &[Player::X, Player::O] {
            //For win by row or column
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
            //For win by diagonal
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
    ///Checks if the move (input) is valid
    pub fn is_valid_move(&self, position: (u8, u8)) -> bool {
        if position.0 as usize >= GRID_SIZE || position.1 as usize >= GRID_SIZE {
            return false;
        }
        self.board[position.0 as usize][position.1 as usize].is_none()
    }
}
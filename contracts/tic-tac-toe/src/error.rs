use cosmwasm_std::StdError;
use thiserror::Error;
use crate::state::GameId;

#[derive(Error, Debug)]
///All contract error messages
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("You Can't Play With YourSelf")]
    YouCantPlayWithYourSelf {},

    #[error("Game Id Not Found")]
    GameIdNotFound { game_id: GameId},

    #[error("Your Session Not Found")]
    YourSessionNotFound {},

    #[error("It's Not Your Turn")]
    ItsNotYourTurn { game_id: GameId},

    #[error("It's Not Your Turn")]
    InviteNotAcceptedYet { game_id: GameId},

    #[error("Invalid Move")]
    InvalidMove { game_id: GameId},

    #[error("Game Over")]
    GameOver { game_id: GameId, status: String, winner: String},

    #[error("Name too short (length {length} min_length {min_length})")]
    NameTooShort { length: u64, min_length: u64 },

    #[error("Name too long (length {length} min_length {max_length})")]
    NameTooLong { length: u64, max_length: u64 },

    #[error("Invalid character(char {c}")]
    InvalidCharacter { c: char },
}

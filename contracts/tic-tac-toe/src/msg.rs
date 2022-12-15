use crate::state::{GameId, Player};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Invite { name: String },
    Accepted { name: String, game_id: GameId },
    Play { position_x: u8, position_y: u8 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GameStatusResponse)]
    GameStatus { game_id: GameId },
    #[returns(BoardStatusResponse)]
    BoardStatus { game_id: GameId },
    #[returns(OpenInvitesResponse)]
    OpenInvites {},
}

#[cw_serde]
pub struct GameStatusResponse {
    pub game_id: GameId,
    pub status: String,
    pub winner: String,
}

#[cw_serde]
pub struct BoardStatusResponse {
    pub board: Vec<Vec<Option<Player>>>,
}

#[cw_serde]
pub struct OpenInvitesResponse {
    pub invites: Vec<GameId>,
}
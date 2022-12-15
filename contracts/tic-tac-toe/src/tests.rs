#[cfg(test)]
mod test_module {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Deps, DepsMut, coins};

    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{BoardStatusResponse, ExecuteMsg, GameStatusResponse, InstantiateMsg, OpenInvitesResponse, QueryMsg};
    use crate::state::{GameId, Player};

    fn mock_init(deps: DepsMut) {
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    fn assert_init_state(deps: Deps, expected: OpenInvitesResponse) {
        let res = query(deps, mock_env(), QueryMsg::OpenInvites {}).unwrap();
        let value: OpenInvitesResponse = from_binary(&res).unwrap();
        assert_eq!(value, expected);
    }

    fn mock_invite(deps: DepsMut, name: String) {
        let info = mock_info(name.as_str(), &[]);
        let msg = ExecuteMsg::Invite {
            name,
        };
        let _res = execute(deps, mock_env(), info, msg)
            .expect("contract successfully handles Invite message");
    }

    fn mock_accepted_invite(deps: DepsMut, name: String, game_id: GameId) {
        let info = mock_info(name.as_str(), &[]);
        let msg = ExecuteMsg::Accepted {
            name,
            game_id
        };
        let _res = execute(deps, mock_env(), info, msg)
            .expect("contract successfully handles Accepted Invite message");
    }

    fn mock_play(deps: DepsMut, name: String, position_x: u8, position_y: u8) {
        let info = mock_info(name.as_str(), &[]);
        let msg = ExecuteMsg::Play {
            position_x,
            position_y
        };
        let _res = execute(deps, mock_env(), info, msg)
            .expect("contract successfully handles Play message");
    }

    fn assert_open_invites(deps: Deps, invites: OpenInvitesResponse) {
        let res = query(
            deps,
            mock_env(),
            QueryMsg::OpenInvites {},
        ).unwrap();
        let value: OpenInvitesResponse = from_binary(&res).unwrap();
        assert_eq!(invites, value);
    }

    fn assert_game_status(deps: Deps, game_id: GameId, game_status: GameStatusResponse) {
        let res = query(
            deps,
            mock_env(),
            QueryMsg::GameStatus { game_id },
        ).unwrap();
        let value: GameStatusResponse = from_binary(&res).unwrap();
        assert_eq!(game_status, value);
    }

    fn assert_board_status(deps: Deps, game_id: GameId, game_status: BoardStatusResponse) {
        let res = query(
            deps,
            mock_env(),
            QueryMsg::BoardStatus { game_id },
        ).unwrap();
        let value: BoardStatusResponse = from_binary(&res).unwrap();
        assert_eq!(game_status, value);
    }
    
    #[test]
    fn proper_init() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        assert_init_state(
            deps.as_ref(),
            OpenInvitesResponse {
                invites: vec![]
            },
        );
    }

    #[test]
    fn invite_with_name_and_query_works() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
    }

    #[test]
    fn accepted_invite_and_query_works() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        mock_accepted_invite(deps.as_mut(), "Bob".to_string(), 1);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game in progress".to_string(),
            winner: "None".to_string()
        });
    }

    #[test]
    fn fails_on_invite_accepted_by_same_inviter() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        let info = mock_info("Alice", &[]);
        let msg = ExecuteMsg::Accepted {
            name: "Alice".to_string(),
            game_id: 1,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::YouCantPlayWithYourSelf {}) => {}
            Err(_) => panic!("Unknown error"),
        }
    }

    #[test]
    fn fails_on_game_id_not_found_on_accepted_invite() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice Wonder".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        let info = mock_info("Bob", &[]);
        let msg = ExecuteMsg::Accepted {
            name: "Bob".to_string(),
            game_id: 2,
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::GameIdNotFound { .. }) => {}
            Err(_) => panic!("Unknown error"),
        }
    }

    #[test]
    fn invite_fails_with_invalid_name() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        let info = mock_info("Alice", &[]);
        // hi is too short
        let msg = ExecuteMsg::Invite {
            name: "hi".to_string(),
        };
        match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::NameTooShort { .. }) => {}
            Err(_) => panic!("Unknown error"),
        }
        // 65 chars is too long
        let msg = ExecuteMsg::Invite {
            name: "01234567890123456789012345678901234567890123456789012345678901234".to_string(),
        };
        match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::NameTooLong { .. }) => {}
            Err(_) => panic!("Unknown error"),
        }
        // no special chars...
        let msg = ExecuteMsg::Invite {
            name: "*LOUD".to_string(),
        };
        match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::InvalidCharacter { c }) => assert_eq!(c, '*'),
            Err(_) => panic!("Unknown error"),
        }
    }

    #[test]
    fn accepted_invite_fails_with_invalid_name() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        let info = mock_info("Alice", &[]);
        // hi is too short
        let msg = ExecuteMsg::Accepted {
            name: "hi".to_string(),
            game_id: 1,
        };
        match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::NameTooShort { .. }) => {}
            Err(_) => panic!("Unknown error"),
        }
        // 65 chars is too long
        let msg = ExecuteMsg::Accepted {
            name: "01234567890123456789012345678901234567890123456789012345678901234".to_string(),
            game_id: 1,
        };
        match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::NameTooLong { .. }) => {}
            Err(_) => panic!("Unknown error"),
        }
        // no special chars...
        let msg = ExecuteMsg::Accepted {
            name: "*LOUD".to_string(),
            game_id: 1,
        };
        match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
            Ok(_) => panic!("Must return error"),
            Err(ContractError::InvalidCharacter { c }) => assert_eq!(c, '*'),
            Err(_) => panic!("Unknown error"),
        }
    }

    #[test]
    fn play_row_win_and_query_works() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        mock_accepted_invite(deps.as_mut(), "Bob".to_string(), 1);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game in progress".to_string(),
            winner: "None".to_string()
        });
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 0);
        mock_play(deps.as_mut(), "Bob".to_string(), 1, 2);
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 1);
        mock_play(deps.as_mut(), "Bob".to_string(), 1, 0);
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 2);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game Over".to_string(),
            winner: "Alice".to_string()
        });
        assert_board_status(deps.as_ref(), 1, BoardStatusResponse{
            board: vec![
                vec![Some(Player::X),Some(Player::X),Some(Player::X)],
                vec![Some(Player::O), None, Some(Player::O)],
                vec![None, None, None]]
        });
    }

    #[test]
    fn play_col_win_and_query_works() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        mock_accepted_invite(deps.as_mut(), "Bob".to_string(), 1);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game in progress".to_string(),
            winner: "None".to_string()
        });
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 0);
        mock_play(deps.as_mut(), "Bob".to_string(), 0, 1);
        mock_play(deps.as_mut(), "Alice".to_string(), 1, 0);
        mock_play(deps.as_mut(), "Bob".to_string(), 0, 2);
        mock_play(deps.as_mut(), "Alice".to_string(), 2, 0);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game Over".to_string(),
            winner: "Alice".to_string()
        });
    }

    #[test]
    fn play_diagonal_win_and_query_works() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        mock_accepted_invite(deps.as_mut(), "Bob".to_string(), 1);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game in progress".to_string(),
            winner: "None".to_string()
        });
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 0);
        mock_play(deps.as_mut(), "Bob".to_string(), 0, 1);
        mock_play(deps.as_mut(), "Alice".to_string(), 1, 1);
        mock_play(deps.as_mut(), "Bob".to_string(), 0, 2);
        mock_play(deps.as_mut(), "Alice".to_string(), 2, 2);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game Over".to_string(),
            winner: "Alice".to_string()
        });
    }

    #[test]
    fn play_draw_and_query_works() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        mock_accepted_invite(deps.as_mut(), "Bob".to_string(), 1);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game in progress".to_string(),
            winner: "None".to_string()
        });
        mock_play(deps.as_mut(), "Alice".to_string(), 2, 2);
        mock_play(deps.as_mut(), "Bob".to_string(), 1, 2);
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 1);
        mock_play(deps.as_mut(), "Bob".to_string(), 2, 0);
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 2);
        mock_play(deps.as_mut(), "Bob".to_string(), 2, 1);
        mock_play(deps.as_mut(), "Alice".to_string(), 1, 0);
        mock_play(deps.as_mut(), "Bob".to_string(), 0, 0);
        mock_play(deps.as_mut(), "Alice".to_string(), 1, 1);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game Over".to_string(),
            winner: "None, Draw".to_string()
        });
    }

    #[test]
    fn play_multiple_session_row_win_and_query_works() {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut());
        mock_invite(deps.as_mut(), "Alice".to_string());
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1] });
        mock_invite(deps.as_mut(), "Dave".to_string());
        // Multiple open invites are available
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![1,2] });

        // Game ids are removed from open invites after invite is accepted by player
        mock_accepted_invite(deps.as_mut(), "Bob".to_string(), 1);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game in progress".to_string(),
            winner: "None".to_string()
        });
        assert_open_invites(deps.as_ref(), OpenInvitesResponse{ invites: vec![2] });
        mock_accepted_invite(deps.as_mut(), "Josh".to_string(), 2);
        assert_game_status(deps.as_ref(), 2, GameStatusResponse{
            game_id: 2,
            status: "Game in progress".to_string(),
            winner: "None".to_string()
        });
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 0);
        mock_play(deps.as_mut(), "Bob".to_string(), 1, 2);
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 1);
        mock_play(deps.as_mut(), "Bob".to_string(), 1, 0);
        mock_play(deps.as_mut(), "Alice".to_string(), 0, 2);
        assert_game_status(deps.as_ref(), 1, GameStatusResponse{
            game_id: 1,
            status: "Game Over".to_string(),
            winner: "Alice".to_string()
        });
        mock_play(deps.as_mut(), "Josh".to_string(), 0, 0);
        mock_play(deps.as_mut(), "Dave".to_string(), 1, 2);
        mock_play(deps.as_mut(), "Josh".to_string(), 0, 1);
        mock_play(deps.as_mut(), "Dave".to_string(), 1, 0);
        mock_play(deps.as_mut(), "Josh".to_string(), 0, 2);
        assert_game_status(deps.as_ref(), 2, GameStatusResponse{
            game_id: 2,
            status: "Game Over".to_string(),
            winner: "Josh".to_string()
        });
    }
}

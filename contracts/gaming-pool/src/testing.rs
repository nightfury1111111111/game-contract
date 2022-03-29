#![allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{Addr};
    use crate::contract::{CLAIMED_REWARD, GAME_CANCELLED, GAME_COMPLETED, GAME_POOL_OPEN, INITIAL_REFUND_AMOUNT, instantiate};
    use crate::execute::{cancel_game, claim_refund, claim_reward, create_pool, game_pool_bid_submit, game_pool_reward_distribute, lock_game, save_team_details, set_platform_fee_wallets, set_pool_type_params};

    use crate::msg::{InstantiateMsg};
    use crate::query::{get_team_count_for_user_in_pool_type, query_game_details, query_pool_details, query_team_details};
    use crate::state::{GameResult, PLATFORM_WALLET_PERCENTAGES, POOL_TEAM_DETAILS, WalletPercentage};

    #[test]
    fn test_create_and_query_game() {
        let mut deps = mock_dependencies(&[]);
        let platform_fee = Uint128::from(300000u128);
        let transaction_fee = Uint128::from(100000u128);
        let _owner1_info = mock_info("Owner001", &[coin(1000, "stake")]);
        let instantiate_msg = InstantiateMsg {
            minting_contract_address: "cwtoken11111".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            transaction_fee: transaction_fee,
            game_id: "Game001".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let queryRes = query_game_details(&mut deps.storage);
        match queryRes {
            Ok(gameDetail) => {
                assert_eq!(gameDetail.game_id, "Game001".to_string());
                assert_eq!(gameDetail.game_status, GAME_POOL_OPEN);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
    }

    #[test]
    fn test_create_and_query_pool_detail() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Owner001", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);
        let transaction_fee = Uint128::from(100000u128);

        let instantiate_msg = InstantiateMsg {
            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            transaction_fee: transaction_fee,
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let rsp = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
        );
        let mut poolId = String::new();

        match rsp {
            Ok(rsp) => {
                poolId = rsp.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let queryRes = query_pool_details(&mut deps.storage, poolId);
        match queryRes {
            Ok(poolDetail) => {
                assert_eq!(poolDetail.game_id, "Game001".to_string());
                assert_eq!(poolDetail.pool_type, "oneToOne".to_string());
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }
    }

    #[test]
    fn test_save_and_query_team_detail() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Owner001", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);
        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let rsp = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
        );
        let mut poolId = String::new();

        match rsp {
            Ok(rsp) => {
                poolId = rsp.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let rsp_save_team = save_team_details(
            &mut deps.storage,
            mock_env(),
            "Gamer001".to_string(),
            poolId.to_string(),
            "Team001".to_string(),
            "Game001".to_string(),
            "oneToOne".to_string(),
            Uint128::from(144262u128),
            false,
            Uint128::from(0u128),
            false,
            100,
            2,
        );

        let mut teamId = String::new();

        match rsp_save_team {
            Ok(rsp_save_team) => {
                teamId = rsp_save_team.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let queryRes = query_team_details(&mut deps.storage, poolId.to_string(), teamId.to_string());
        match queryRes {
            Ok(poolTeamDetail) => {
                assert_eq!(poolTeamDetail.pool_id, poolId.to_string());
                //assert_eq!(gameDetail.game_status, GAME_POOL_OPEN);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }
    }

    #[test]
    fn test_get_team_count_for_user_in_pool_type() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Owner001", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);
        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let rsp = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
        );
        let mut poolId = String::new();

        match rsp {
            Ok(rsp) => {
                poolId = rsp.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let rsp_save_team_1 = save_team_details(
            &mut deps.storage,
            mock_env(),
            "Gamer001".to_string(),
            poolId.to_string(),
            "Team001".to_string(),
            "Game001".to_string(),
            "oneToOne".to_string(),
            Uint128::from(144262u128),
            false,
            Uint128::from(0u128),
            false,
            100,
            2,
        );
        let rsp_save_team_2 = save_team_details(
            &mut deps.storage,
            mock_env(),
            "Gamer001".to_string(),
            poolId.to_string(),
            "Team002".to_string(),
            "Game001".to_string(),
            "oneToOne".to_string(),
            Uint128::from(144262u128),
            false,
            Uint128::from(0u128),
            false,
            100,
            2,
        );
        let rsp_save_team_3 = save_team_details(
            &mut deps.storage,
            mock_env(),
            "Gamer001".to_string(),
            poolId.to_string(),
            "Team003".to_string(),
            "Game001".to_string(),
            "oneToOne".to_string(),
            Uint128::from(144262u128),
            false,
            Uint128::from(0u128),
            false,
            100,
            2,
        );

        let team_count = get_team_count_for_user_in_pool_type(
            &mut deps.storage,
            "Gamer001".to_string(),
            "Game001".to_string(),
            "oneToOne".to_string(),
        );

        match team_count {
            Ok(team_count) => {
                assert_eq!(team_count, 3);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }
    }

    #[test]
    fn test_game_pool_bid_submit_when_pool_team_in_range() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer001", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();

        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            2,
            rake_list,
        );

        let rsp = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
        );
        let mut poolId = String::new();

        match rsp {
            Ok(rsp) => {
                poolId = rsp.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let rewardInfo = mock_info("rewardInfo", &[]);
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            poolId.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        let queryRes = query_pool_details(&mut deps.storage, "1".to_string());
        match queryRes {
            Ok(poolDetail) => {
                assert_eq!(poolDetail.pool_id, "1".to_string());
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }
    }

    #[test]
    fn test_game_pool_bid_submit_when_pool_team_not_in_range() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer001", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();

        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
            Uint128::from(144262u128),
            1,
            1,
            1,
            rake_list,
        );

        let rsp = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
        );
        let mut poolId = String::new();

        match rsp {
            Ok(rsp) => {
                poolId = rsp.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let rewardInfo = mock_info("rewardInfo", &[]);
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            poolId.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            owner1_info.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            poolId.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        let queryRes = query_pool_details(&mut deps.storage, "2".to_string());
        match queryRes {
            Ok(poolDetail) => {
                // there should not be any pool with id 2
                assert_eq!(1, 2);
            }
            Err(e) => {
                // there should not be any pool with id 2
                assert_eq!(1, 1);
            }
        }
    }

    #[test]
    fn test_crete_different_pool_type_and_add_multiple_game_for_given_user() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer001", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();

        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            10,
            rake_list.clone(),
        );
        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "multiple".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            10,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let mut pool_id_2 = String::new();
        let rsp_2 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "multiple".to_string(),
        );
        match rsp_2 {
            Ok(rsp_2) => {
                pool_id_2 = rsp_2.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        // create  pool with same pool type as in pool_id_1
        let mut pool_id_3 = String::new();
        let rsp_3 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToOne".to_string(),
        );
        match rsp_3 {
            Ok(rsp_3) => {
                pool_id_3 = rsp_3.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        let rewardInfo = mock_info("rewardInfo", &[]);
        // Adding multile team to pool_1 for Game001
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                assert_eq!(pool_detail_1.current_teams_count, 4u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(4, 5);
            }
        }

        // Adding multile team to pool_2 for Game001. some of team is already added in pool_1
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "multiple".to_string(),
            pool_id_2.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "multiple".to_string(),
            pool_id_2.to_string(),
            "Team004".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "multiple".to_string(),
            pool_id_2.to_string(),
            "Team005".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_2 = query_pool_details(&mut deps.storage, pool_id_2.to_string());
        match query_pool_details_2 {
            Ok(pool_detail_2) => {
                assert_eq!(pool_detail_2.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(5, 6);
            }
        }

        // Adding same team to another pool of same pool type
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "oneToOne".to_string(),
            pool_id_3.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer001".to_string(),
            "multiple".to_string(),
            pool_id_3.to_string(),
            "Team004".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        let query_pool_details_3 = query_pool_details(&mut deps.storage, pool_id_3.to_string());
        match query_pool_details_3 {
            Ok(pool_detail_3) => {
                assert_eq!(pool_detail_3.current_teams_count, 2u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(6, 7);
            }
        }
    }

    #[test]
    fn test_max_team_per_pool_type_for_given_user() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            2,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 2u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }
    }

    #[test]
    fn test_game_pool_reward_distribute() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(100u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let lock_game_rsp = lock_game(deps.as_mut(), mock_env(), adminInfo.clone());
        match lock_game_rsp {
            Ok(lock_game_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    lock_game_rsp.attributes[1].value.clone(),
                    "GAME_POOL_CLOSED".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        let game_pool_reward_distribute_rsp = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results,
        );

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {}
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(4, 5);
            }
        }

        let query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_COMPLETED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
        for team in team_details {
            assert_eq!(team[0].reward_amount, Uint128::from(100u128));
            assert_eq!(team[1].reward_amount, Uint128::from(200u128));
            assert_eq!(team[2].reward_amount, Uint128::from(300u128));
        }
    }

    #[test]
    fn test_claim_refund() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,
            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let cancelInfo = mock_info("cancelInfo", &[]);
        let cancel_rsp = cancel_game(deps.as_mut(), mock_env(), adminInfo.clone());

/*
		23 Mar 2022, commenting this out because call to proxy cannot be made 
		it succeeds till calculating refund amount = 444262

        let claim_refund_rsp = claim_refund(deps.as_mut(), owner1_info.clone(), "Gamer002".to_string(), mock_env());
        match claim_refund_rsp {
            Ok(claim_refund_rsp) => {
                let amt = claim_refund_rsp.attributes[0].value.clone();
                let expamt = Uint128::from(144262u128) + platform_fee;
                let expamtStr = expamt.to_string();
                assert_eq!(amt, expamtStr);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(5, 6);
            }
        }
*/
    }

    #[test]
    fn test_cancel_game() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(100u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let lock_game_rsp = lock_game(deps.as_mut(), mock_env(), adminInfo.clone());
        match lock_game_rsp {
            Ok(lock_game_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    lock_game_rsp.attributes[1].value.clone(),
                    "GAME_POOL_CLOSED".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        let cancelInfo = mock_info("cancelInfo", &[]);
        let game_pool_reward_distribute_rsp =
            cancel_game(deps.as_mut(), mock_env(), adminInfo.clone());

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {}
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(4, 5);
            }
        }

        let query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_CANCELLED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(5, 6);
            }
        }
        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
        for team in team_details {
            assert_eq!(team[0].reward_amount, Uint128::zero());
            assert_eq!(team[1].reward_amount, Uint128::zero());
            assert_eq!(team[2].reward_amount, Uint128::zero());
        }
    }

    #[test]
    fn test_claim_reward() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let platform_fee = Uint128::from(300000u128);
        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            rewardInfo.clone(),
            instantiate_msg,
        );
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(500u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let lock_game_rsp = lock_game(deps.as_mut(), mock_env(), adminInfo.clone());
        match lock_game_rsp {
            Ok(lock_game_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    lock_game_rsp.attributes[1].value.clone(),
                    "GAME_POOL_CLOSED".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        let game_pool_reward_distribute_rsp = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results,
        );

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {}
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(4, 5);
            }
        }

        let mut query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_COMPLETED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(5, 6);
            }
        }
        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
        for team in team_details {
            assert_eq!(team[0].reward_amount, Uint128::from(500u128));
            assert_eq!(team[1].reward_amount, Uint128::from(200u128));
            assert_eq!(team[2].reward_amount, Uint128::from(300u128));
        }

/*
		23 Mar 2022, commenting this out because call to proxy cannot be made 
		it succeeds till calculating reward amount = 1000

        let claim_reward_rsp =
            claim_reward(deps.as_mut(), owner1_info.clone(), "Gamer002".to_string(), mock_env());
        match claim_reward_rsp {
            Ok(claim_reward_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    claim_reward_rsp.attributes[0].value.clone(),
                    "1000".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(6, 7);
            }
        }
*/

        query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_COMPLETED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(7, 8);
            }
        }

        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
        for team in team_details {
            assert_eq!(team[0].reward_amount, Uint128::from(500u128)); // TODO This reward should be 0 after full functionality working.
            assert_eq!(team[1].reward_amount, Uint128::from(200u128)); // TODO This reward should be 0 after full functionality working.
            assert_eq!(team[2].reward_amount, Uint128::from(300u128)); // TODO This reward should be 0 after full functionality working.
/*
			23 Mar 2022, commenting this out because call to proxy cannot be made 
            assert_eq!(team[0].claimed_reward, CLAIMED_REWARD);
            assert_eq!(team[1].claimed_reward, CLAIMED_REWARD);
            assert_eq!(team[2].claimed_reward, CLAIMED_REWARD);
*/
        }
    }

    #[test]
    fn test_claim_reward_twice() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(100u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let lock_game_rsp = lock_game(deps.as_mut(), mock_env(), adminInfo.clone());
        match lock_game_rsp {
            Ok(lock_game_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    lock_game_rsp.attributes[1].value.clone(),
                    "GAME_POOL_CLOSED".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        let game_pool_reward_distribute_rsp = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results,
        );

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {}
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(4, 5);
            }
        }

        let mut query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_COMPLETED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(5, 6);
            }
        }
        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
        for team in team_details {
            assert_eq!(team[0].reward_amount, Uint128::from(100u128));
            assert_eq!(team[1].reward_amount, Uint128::from(200u128));
            assert_eq!(team[2].reward_amount, Uint128::from(300u128));
        }

/*
		23 Mar 2022, commenting this out because call to proxy cannot be made 
		it succeeds till calculating reward amount = 600

        let claim_reward_rsp =
            claim_reward(deps.as_mut(), owner1_info.clone(), "Gamer002".to_string(), mock_env());
        match claim_reward_rsp {
            Ok(claim_reward_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    claim_reward_rsp.attributes[0].value.clone(),
                    "600".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(6, 7);
            }
        }
*/
        query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_COMPLETED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(7, 8);
            }
        }

        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
        for team in team_details {
            assert_eq!(team[0].reward_amount, Uint128::from(100u128)); // TODO This reward should be 0 after full functionality working.
            assert_eq!(team[1].reward_amount, Uint128::from(200u128)); // TODO This reward should be 0 after full functionality working.
            assert_eq!(team[2].reward_amount, Uint128::from(300u128)); // TODO This reward should be 0 after full functionality working.
/*
			23 Mar 2022, commenting this out because call to proxy cannot be made 
            assert_eq!(team[0].claimed_reward, CLAIMED_REWARD);
            assert_eq!(team[1].claimed_reward, CLAIMED_REWARD);
            assert_eq!(team[2].claimed_reward, CLAIMED_REWARD);
*/
        }

/*
			23 Mar 2022, commenting this out because call to proxy cannot be made 
        let claim_reward_rsp_2 =
            claim_reward(deps.as_mut(), owner1_info.clone(), "Gamer002".to_string(), mock_env());
        match claim_reward_rsp_2 {
            Ok(claim_reward_rsp_2) => {
                // IT should not come here
                assert_eq!(1, 2);
            }
            Err(e) => {
                let outstr = format!("error parsing header: {:?}", e);
                println!("{:?}", outstr);
                assert_eq!(
                    outstr,
                    "error parsing header: Std(GenericErr { msg: \"No reward for this user\" })"
                );
            }
        }
*/
    }

    #[test]
    fn test_refund_game_pool_close_with_team_less_than_minimum_team_count() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            10,
            20,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let platform_fee = Uint128::from(300000u128);
        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(100u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let lock_game_rsp = lock_game(deps.as_mut(), mock_env(), adminInfo.clone());
        match lock_game_rsp {
            Ok(lock_game_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    lock_game_rsp.attributes[1].value.clone(),
                    "GAME_POOL_CLOSED".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }
        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
		let mut teams = Vec::new();
        match team_details {
            Ok(some_teams) => {
                teams = some_teams;
            }
            Err(e) => {}
        }

		let mut count = 0;
        for team in teams {
			count += 1;
			println!("team = {:?}", team);
        }
		assert_eq!(count,3);
    }

    #[test]
    fn test_cancel_on_completed_game() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(30u128);

        let transaction_fee = Uint128::from(10u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,
            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let rewardInfo = mock_info("rewardInfo", &[]);

        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        println!("This is the value for the  pool_details{:?}", query_pool_details_1);
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(100u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let lock_game_rsp = lock_game(deps.as_mut(), mock_env(), adminInfo.clone());
        match lock_game_rsp {
            Ok(lock_game_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    lock_game_rsp.attributes[1].value.clone(),
                    "GAME_POOL_CLOSED".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        let game_pool_reward_distribute_rsp = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results,
        );

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {}
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(4, 5);
            }
        }

        let mut query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_COMPLETED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(5, 6);
            }
        }

        let game_cancel_rsp = cancel_game(deps.as_mut(), mock_env(), adminInfo.clone());

        match game_cancel_rsp {
            Ok(game_cancel_rsp) => {
                assert_eq!(6, 7);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(
                    e.to_string(),
                    "Generic error: Cant cancel game as it is already over".to_string()
                );
            }
        }
    }

    #[test]
    fn test_reward_distribute_non_completed_game() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(100u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let mut game_pool_reward_distribute_rsp = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results.clone(),
        );

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {
                assert_eq!(2, 3);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(
                    e.to_string(),
                    "Generic error: Rewards cant be distributed as game not yet started"
                        .to_string()
                );
            }
        }

        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        game_pool_reward_distribute_rsp = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results,
        );

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {
                assert_eq!(4, 5);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(
                    e.to_string(),
                    "Generic error: Rewards cant be distributed as game not yet started"
                        .to_string()
                );
            }
        }
    }

    #[test]
    fn test_game_pool_reward_distribute_again() {
        let mut deps = mock_dependencies(&[]);
        let owner1_info = mock_info("Gamer002", &[coin(1000, "stake")]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let adminInfo = mock_info("admin11111", &[]);
        instantiate(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            instantiate_msg,
        );

        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_pool_type_params(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
            Uint128::from(144262u128),
            2,
            10,
            5,
            rake_list.clone(),
        );

        // create multiple pool
        let mut pool_id_1 = String::new();
        let rsp_1 = create_pool(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            "oneToTwo".to_string(),
        );
        match rsp_1 {
            Ok(rsp_1) => {
                pool_id_1 = rsp_1.attributes[0].value.clone();
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(1, 2);
            }
        }
        let platform_fee = Uint128::from(300000u128);
        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };
        let rewardInfo = mock_info("rewardInfo", &[]);
        let ownerXInfo = mock_info("cwtoken11111", &[coin(1000, "stake")]);
        // Adding same team twice in same pool
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team001".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team002".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );
        game_pool_bid_submit(
            deps.as_mut(),
            mock_env(),
            ownerXInfo.clone(),
            "Gamer002".to_string(),
            "oneToTwo".to_string(),
            pool_id_1.to_string(),
            "Team003".to_string(),
            Uint128::from(144262u128) + platform_fee,
            true,
        );

        let query_pool_details_1 = query_pool_details(&mut deps.storage, pool_id_1.to_string());
        match query_pool_details_1 {
            Ok(pool_detail_1) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                assert_eq!(pool_detail_1.current_teams_count, 3u32);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(2, 3);
            }
        }

        let game_result_1 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team001".to_string(),
            team_rank: 1u64,
            team_points: 100u64,
            reward_amount: Uint128::from(100u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_2 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team002".to_string(),
            team_rank: 2u64,
            team_points: 200u64,
            reward_amount: Uint128::from(200u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let game_result_3 = GameResult {
            gamer_address: "Gamer002".to_string(),
            game_id: "Game001".to_string(),
            team_id: "Team003".to_string(),
            team_rank: 2u64,
            team_points: 300u64,
            reward_amount: Uint128::from(300u128),
            refund_amount: Uint128::from(INITIAL_REFUND_AMOUNT),
        };
        let mut game_results: Vec<GameResult> = Vec::new();
        game_results.push(game_result_1);
        game_results.push(game_result_2);
        game_results.push(game_result_3);

        let lock_game_rsp = lock_game(deps.as_mut(), mock_env(), adminInfo.clone());
        match lock_game_rsp {
            Ok(lock_game_rsp) => {
                //Since max allowed team for gamer under this pooltype is 2 so it will not allow 3rd team creation under this pooltype.
                //assert_eq!(pool_detail_1.current_teams_count, 3u32);
                assert_eq!(
                    lock_game_rsp.attributes[1].value.clone(),
                    "GAME_POOL_CLOSED".to_string()
                );
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(3, 4);
            }
        }

        let game_pool_reward_distribute_rsp = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results.clone(),
        );

        match game_pool_reward_distribute_rsp {
            Ok(game_pool_reward_distribute_rsp) => {}
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(4, 5);
            }
        }

        let query_game_status_res = query_game_details(&mut deps.storage);
        match query_game_status_res {
            Ok(query_game_status_res) => {
                assert_eq!(query_game_status_res.game_status, GAME_COMPLETED);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(5, 6);
            }
        }
        let team_details = POOL_TEAM_DETAILS.load(&mut deps.storage, pool_id_1.clone());
        for team in team_details {
            assert_eq!(team[0].reward_amount, Uint128::from(100u128));
            assert_eq!(team[1].reward_amount, Uint128::from(200u128));
            assert_eq!(team[2].reward_amount, Uint128::from(300u128));
        }

        let game_pool_reward_distribute_rsp_2 = game_pool_reward_distribute(
            deps.as_mut(),
            mock_env(),
            adminInfo.clone(),
            pool_id_1.to_string(),
            game_results,
        );

        match game_pool_reward_distribute_rsp_2 {
            Ok(game_pool_reward_distribute_rsp_2) => {
                assert_eq!(6, 7);
            }
            Err(e) => {
                println!("error parsing header: {:?}", e);
                assert_eq!(
                    e.to_string(),
                    "Generic error: Rewards are already distributed for this pool".to_string()
                );
            }
        }
    }

    #[test]
    fn test_set_platform_fee_wallets() {
        let mut deps = mock_dependencies(&[]);
        let platform_fee = Uint128::from(300000u128);

        let transaction_fee = Uint128::from(100000u128);
        let instantiate_msg = InstantiateMsg {
            transaction_fee: transaction_fee,

            minting_contract_address: "cwtoken11111".to_string(),
            admin_address: "admin11111".to_string(),
            platform_fee: platform_fee,
            game_id: "Game001".to_string(),
            platform_fees_collector_wallet: "FEE_WALLET".to_string(),
            astro_proxy_address: "ASTROPORT".to_string(),
        };

        let adminInfo = mock_info("admin11111", &[]);
        let mut rake_list: Vec<WalletPercentage> = Vec::new();
        let rake_1 = WalletPercentage {
            wallet_address: "rake_1".to_string(),
            wallet_name: "rake_1".to_string(),
            percentage: 1u32,
        };
        rake_list.push(rake_1);
        let rake_2 = WalletPercentage {
            wallet_address: "rake_2".to_string(),
            wallet_name: "rake_2".to_string(),
            percentage: 2u32,
        };
        rake_list.push(rake_2);

        let rake_3 = WalletPercentage {
            wallet_address: "rake_3".to_string(),
            wallet_name: "rake_3".to_string(),
            percentage: 3u32,
        };
        rake_list.push(rake_3);

        set_platform_fee_wallets(deps.as_mut(), adminInfo, rake_list);

        let wallets = PLATFORM_WALLET_PERCENTAGES.load(&mut deps.storage, "test".to_string());

        for wallet in wallets {
            assert_eq!(wallet.wallet_name, "rake_1".to_string());
            assert_eq!(wallet.wallet_name, "rake_2".to_string());
            assert_eq!(wallet.wallet_name, "rake_3".to_string());
        }
    }
}


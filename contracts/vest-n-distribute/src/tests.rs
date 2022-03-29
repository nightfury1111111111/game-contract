
#[cfg(test)]
mod tests {
    // use crate::msg::InstantiateMsg;
    use crate::execute::{calculate_tokens_for_this_period, distribute_vested};
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{Addr, Deps, Uint128};
    use crate::query::query_balance;
    use crate::state::VestingDetails;

    use super::*;

    fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
        query_balance(deps, address.into()).unwrap().balance
    }

    #[test]
    fn vesting_test_cases() {
        assert_eq!(1, 1);
    }

    // this will set up the instantiation for other tests
    // fn do_instantiate(deps: DepsMut, addr: &str, amount: Uint128) -> TokenInfoResponse {
    //     _do_instantiate(deps, addr, amount, None)
    // }

    // this will set up the instantiation for other tests
    // fn _do_instantiate(
    //     mut deps: DepsMut,
    //     addr: &str,
    //     amount: Uint128,
    //     mint: Option<MinterResponse>,
    // ) -> TokenInfoResponse {
    //     let instantiate_msg = InstantiateMsg {
    //         admin_wallet: Addr::unchecked("terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k"),
    //         fury_token_contract: Addr::unchecked("terra1ttjw6nscdmkrx3zhxqx3md37phldgwhggm345k"),
    //         vesting: {

    //         },
    //     };
    //     //WIP
    //     let info = mock_info("creator", &[]);
    //     let env = mock_env();
    //     let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     let meta = query_token_info(deps.as_ref()).unwrap();
    //     assert_eq!(
    //         meta,
    //         TokenInfoResponse {
    //             name: "Auto Gen".to_string(),
    //             symbol: "AUTO".to_string(),
    //             decimals: 3,
    //             total_supply: amount,
    //         }
    //     );
    //     assert_eq!(get_balance(deps.as_ref(), addr), amount);
    //     // assert_eq!(query_minter(deps.as_ref()).unwrap(), mint,);
    //     meta
    // }

    #[test]
    fn transfer_to_categories() {
        let mut deps = mock_dependencies(&[]);
        let distribute_from = String::from("addr0001");
        let distribute_to = String::from("addr0002");
        let amount = Uint128::from(1000u128);

        // do_instantiate(deps.as_mut(), &distribute_from, amount);

        let init_from_balance = get_balance(deps.as_ref(), distribute_from.clone());
        let init_to_balance = get_balance(deps.as_ref(), distribute_to.clone());

        // Transfer the funds
        let mut_deps = &mut deps.as_mut();
        let _res = distribute_vested(
            mut_deps,
            distribute_from.clone(),
            distribute_to.clone(),
            amount,
        );

        let calc_new_from_balance = init_from_balance - amount;
        let calc_new_to_balance = init_to_balance + amount;

        let new_from_balance = get_balance(deps.as_ref(), distribute_from);
        let new_to_balance = get_balance(deps.as_ref(), distribute_to);
        // check that the transfer happened
        assert_eq!(calc_new_from_balance, new_from_balance);
        assert_eq!(calc_new_to_balance, new_to_balance);
    }

    #[test]
    fn fail_transfer_to_categories() {
        let mut deps = mock_dependencies(&[]);
        let distribute_from = String::from("addr0001");
        let distribute_to = String::from("addr0002");
        let _amount1 = Uint128::from(1000u128);

        // do_instantiate(deps.as_mut(), &distribute_from, amount1);

        let init_from_balance = get_balance(deps.as_ref(), distribute_from.clone());
        let init_to_balance = get_balance(deps.as_ref(), distribute_to.clone());

        let amount = init_from_balance + Uint128::from(1000u128);

        // Try to transfer more than the funds available - it should fail
        let mut_deps = &mut deps.as_mut();
        let _res = distribute_vested(
            mut_deps,
            distribute_from.clone(),
            distribute_to.clone(),
            amount,
        );

        let new_from_balance = get_balance(deps.as_ref(), distribute_from);
        let new_to_balance = get_balance(deps.as_ref(), distribute_to);

        // check that the transfer did not happen
        assert_eq!(new_from_balance, init_from_balance);
        assert_eq!(new_to_balance, init_to_balance);
    }

    fn get_vesting_details() -> VestingDetails {
        let now = mock_env().block.time;
        let category_address = String::from("addr0002");
        return VestingDetails {
            vesting_start_timestamp: now,
            initial_vesting_count: Uint128::zero(),
            initial_vesting_consumed: Uint128::zero(),
            vesting_periodicity: 300, // In seconds
            vesting_count_per_period: Uint128::from(10u128),
            total_vesting_token_count: Uint128::from(2000u128),
            total_claimed_tokens_till_now: Uint128::zero(),
            last_claimed_timestamp: None,
            tokens_available_to_claim: Uint128::zero(),
            last_vesting_timestamp: None,
            cliff_period: 0, // in months
            parent_category_address: Some(category_address),
            should_transfer: true,
        };
    }

    #[test]
    fn test_vesting_at_tge() {
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today
        println!("now {:?}", now);

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.tokens_available_to_claim += vesting_details.vesting_count_per_period;
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed() {
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(1000u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_first_interval() {
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity);
        let vcpp = vesting_details.vesting_count_per_period.u128();
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount.u128(), vcpp);
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_first_interval() {
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(1010u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_2_uncalc_interval() {
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(20u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_2_uncalc_interval() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(1020u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1vested_1uncalc_interval() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();

        vesting_details.tokens_available_to_claim = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp =
            now.minus_seconds(vesting_details.vesting_periodicity * 2);

        vesting_details.last_vesting_timestamp =
            Some(now.minus_seconds(vesting_details.vesting_periodicity));

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(10u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1claimed_1uncalc_interval() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();

        vesting_details.total_claimed_tokens_till_now = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp =
            now.minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);

        vesting_details.last_vesting_timestamp = Some(
            now.minus_seconds(vesting_details.vesting_periodicity)
                .minus_seconds(5),
        );

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(10u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1claimed_half_uncalc_interval() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();

        vesting_details.total_claimed_tokens_till_now = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp =
            now.minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);

        vesting_details.last_vesting_timestamp = Some(
            now.minus_seconds(vesting_details.vesting_periodicity)
                .minus_seconds(5),
        );

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(10u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today
        println!("now {:?}", now);

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.tokens_available_to_claim += vesting_details.vesting_count_per_period;
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::zero());
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_first_interval_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity);
        let _vcpp = vesting_details.vesting_count_per_period.u128();
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount.u128(), 0u128);
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_first_interval_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_2_uncalc_interval_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_2_uncalc_intervalwith_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1vested_1uncalc_interval_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.tokens_available_to_claim = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp =
            now.minus_seconds(vesting_details.vesting_periodicity * 2);

        vesting_details.last_vesting_timestamp =
            Some(now.minus_seconds(vesting_details.vesting_periodicity));

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1claimed_1uncalc_interval_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.total_claimed_tokens_till_now = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp =
            now.minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);

        vesting_details.last_vesting_timestamp = Some(
            now.minus_seconds(vesting_details.vesting_periodicity)
                .minus_seconds(5),
        );

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1claimed_half_uncalc_interval_with_cliff() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        vesting_details.total_claimed_tokens_till_now = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp =
            now.minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);

        vesting_details.last_vesting_timestamp = Some(
            now.minus_seconds(vesting_details.vesting_periodicity)
                .minus_seconds(5),
        );

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today
        println!("now {:?}", now);

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;
        vesting_details.tokens_available_to_claim += vesting_details.vesting_count_per_period;
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(0u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today
        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(1000u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_first_interval_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity);
        let vcpp = vesting_details.vesting_count_per_period.u128();
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount.u128(), vcpp);
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_first_interval_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(1010u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_2_uncalc_interval_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(20u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_with_initial_seed_2_uncalc_interval_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        // vesting at TGE
        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;
        vesting_details.initial_vesting_count = Uint128::from(1000u128);
        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);
        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(1020u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1vested_1uncalc_interval_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;

        vesting_details.tokens_available_to_claim = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);

        vesting_details.last_vesting_timestamp =
            Some(now.minus_seconds(vesting_details.vesting_periodicity));

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(10u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1claimed_1uncalc_interval_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;

        vesting_details.total_claimed_tokens_till_now = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);

        vesting_details.last_vesting_timestamp = Some(
            now.minus_seconds(vesting_details.vesting_periodicity)
                .minus_seconds(5),
        );

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(10u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting_at_tge_no_initial_seed_1claimed_half_uncalc_interval_with_cliff_period_over() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let category_address = String::from("addr0002");

        let now = mock_env().block.time; // today

        let mut vesting_details = get_vesting_details();
        vesting_details.cliff_period = 1;
        let vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(30 * 24 * 60 * 60);
        vesting_details.vesting_start_timestamp = vesting_start_timestamp;

        vesting_details.total_claimed_tokens_till_now = Uint128::from(10u128);

        vesting_details.vesting_start_timestamp = vesting_details
            .vesting_start_timestamp
            .minus_seconds(vesting_details.vesting_periodicity * 2);
        vesting_details.vesting_start_timestamp =
            vesting_details.vesting_start_timestamp.minus_seconds(5);

        vesting_details.last_vesting_timestamp = Some(
            now.minus_seconds(vesting_details.vesting_periodicity)
                .minus_seconds(5),
        );

        let vested_amount = calculate_tokens_for_this_period(
            Addr::unchecked(category_address.clone()),
            now,
            vesting_details,
        );
        match vested_amount {
            Ok(va) => {
                assert_eq!(va.amount, Uint128::from(10u128));
            }
            Err(e) => {
                println!("error = {:?}", e);
                assert_eq!(1, 0);
            }
        }
    }

    #[test]
    fn test_vesting() {
        // use std::time::{Duration, SystemTime, UNIX_EPOCH};
        let _spender_address = String::from("addr0001");
        let _category_address = String::from("addr0002");

        let _now = mock_env().block.time; // today

        let mut _vesting_details = get_vesting_details();

        // vesting_periodicity = 86400; // in seconds
        // vesting_started_before = 92; // in days
        // cliff_period = 3; // in months
        // vesting_start_timestamp = mock_env()
        //     .block
        //     .time
        //     .minus_seconds(vesting_started_before * 86400);
        // last_vesting_timestamp = mock_env().block.time;
        // total_vesting_token_count = Uint128::from(200u128);
        // total_claimed_tokens_till_now = Uint128::from(0u128);
        // tokens_available_to_claim = Uint128::from(10000u128);
        // let vested_amount = calculate_tokens_for_this_period(
        //     Addr::unchecked(category_address.clone()),
        //     now,
        //     VestingDetails {
        //         vesting_start_timestamp: vesting_start_timestamp,
        //         initial_vesting_count: initial_vesting_count,
        //         initial_vesting_consumed: initial_vesting_consumed,
        //         vesting_periodicity: vesting_periodicity,
        //         vesting_count_per_period: vesting_count_per_period,
        //         total_vesting_token_count: total_vesting_token_count,
        //         total_claimed_tokens_till_now: total_claimed_tokens_till_now,
        //         last_claimed_timestamp: last_claimed_timestamp,
        //         tokens_available_to_claim: tokens_available_to_claim,
        //         last_vesting_timestamp: last_vesting_timestamp,
        //         cliff_period: cliff_period,
        //         category_address: Some(category_address.clone()),
        //     },
        // );
        // match vested_amount {
        //     Ok(va) => {
        //         assert_eq!(va.amount, Uint128::from(200u128));
        //     }
        //     Err(e) => {
        //         assert_eq!(1, 0);
        //     }
        // }

        // vesting_periodicity = 86400; // in seconds
        // vesting_started_before = 90; // in days
        // cliff_period = 3; // in months
        // vesting_start_timestamp = mock_env()
        //     .block
        //     .time
        //     .minus_seconds(vesting_started_before * 86400);
        // last_vesting_timestamp = mock_env().block.time;
        // total_vesting_token_count = Uint128::from(200u128);
        // total_claimed_tokens_till_now = Uint128::from(0u128);
        // tokens_available_to_claim = Uint128::from(10000u128);
        // let vested_amount = calculate_tokens_for_this_period(
        //     Addr::unchecked(category_address.clone()),
        //     now,
        //     VestingDetails {
        //         vesting_start_timestamp: vesting_start_timestamp,
        //         initial_vesting_count: initial_vesting_count,
        //         initial_vesting_consumed: initial_vesting_consumed,
        //         vesting_periodicity: vesting_periodicity,
        //         vesting_count_per_period: vesting_count_per_period,
        //         total_vesting_token_count: total_vesting_token_count,
        //         total_claimed_tokens_till_now: total_claimed_tokens_till_now,
        //         last_claimed_timestamp: last_claimed_timestamp,
        //         tokens_available_to_claim: tokens_available_to_claim,
        //         last_vesting_timestamp: last_vesting_timestamp,
        //         cliff_period: cliff_period,
        //         category_address: Some(category_address.clone()),
        //     },
        // );
        // match vested_amount {
        //     Ok(va) => {
        //         assert_eq!(va.amount, Uint128::zero());
        //     }
        //     Err(e) => {
        //         assert_eq!(1, 0);
        //     }
        // }

        // vesting_periodicity = 86400; // in seconds
        // vesting_started_before = 89; // in days
        // cliff_period = 3; // in months
        // let mut vesting_start_timestamp = mock_env()
        //     .block
        //     .time
        //     .minus_seconds(vesting_started_before * 86400);
        // last_vesting_timestamp = mock_env().block.time;
        // total_vesting_token_count = Uint128::from(200u128);
        // total_claimed_tokens_till_now = Uint128::from(0u128);
        // tokens_available_to_claim = Uint128::from(10000u128);
        // let vested_amount = calculate_tokens_for_this_period(
        //     Addr::unchecked(category_address.clone()),
        //     now,
        //     VestingDetails {
        //         vesting_start_timestamp: vesting_start_timestamp,
        //         initial_vesting_count: initial_vesting_count,
        //         initial_vesting_consumed: initial_vesting_consumed,
        //         vesting_periodicity: vesting_periodicity,
        //         vesting_count_per_period: vesting_count_per_period,
        //         total_vesting_token_count: total_vesting_token_count,
        //         total_claimed_tokens_till_now: total_claimed_tokens_till_now,
        //         last_claimed_timestamp: last_claimed_timestamp,
        //         tokens_available_to_claim: tokens_available_to_claim,
        //         last_vesting_timestamp: last_vesting_timestamp,
        //         cliff_period: cliff_period,
        //         category_address: Some(category_address.clone()),
        //     },
        // );
        // match vested_amount {
        //     Ok(va) => {
        //         assert_eq!(va.amount, Uint128::zero());
        //     }
        //     Err(e) => {
        //         assert_eq!(1, 0);
        //     }
        // }

        // vesting_periodicity = 86400; // in seconds
        // vesting_started_before = 89; // in days
        // cliff_period = 0; // in months
        // let mut vesting_start_seconds =
        //     mock_env().block.time.seconds() - vesting_started_before * 86400;
        // last_vesting_timestamp = mock_env().block.time;
        // total_vesting_token_count = Uint128::from(200u128);
        // total_claimed_tokens_till_now = Uint128::from(0u128);
        // tokens_available_to_claim = Uint128::from(10000u128);
        // let vested_amount = calculate_tokens_for_this_period(
        //     Addr::unchecked(category_address.clone()),
        //     now,
        //     VestingDetails {
        //         vesting_start_timestamp: vesting_start_timestamp,
        //         initial_vesting_count: initial_vesting_count,
        //         initial_vesting_consumed: initial_vesting_consumed,
        //         vesting_periodicity: vesting_periodicity,
        //         vesting_count_per_period: vesting_count_per_period,
        //         total_vesting_token_count: total_vesting_token_count,
        //         total_claimed_tokens_till_now: total_claimed_tokens_till_now,
        //         last_claimed_timestamp: last_claimed_timestamp,
        //         tokens_available_to_claim: tokens_available_to_claim,
        //         last_vesting_timestamp: last_vesting_timestamp,
        //         cliff_period: cliff_period,
        //         category_address: Some(category_address.clone()),
        //     },
        // );
        // match vested_amount {
        //     Ok(va) => {
        //         assert_eq!(va.amount, Uint128::from(8900u128));
        //     }
        //     Err(e) => {
        //         assert_eq!(1, 0);
        //     }
        // }

        // vesting_periodicity = 0; // in seconds - immediately vest
        // vesting_started_before = 89; // in days
        // cliff_period = 0; // in months
        // vesting_start_seconds = mock_env().block.time.seconds() - vesting_started_before * 86400;
        // last_vesting_timestamp = mock_env().block.time;
        // total_vesting_token_count = Uint128::from(200u128);
        // total_claimed_tokens_till_now = Uint128::from(0u128);
        // tokens_available_to_claim = Uint128::from(10000u128);
        // let vested_amount = calculate_tokens_for_this_period(
        //     Addr::unchecked(category_address.clone()),
        //     now,
        //     VestingDetails {
        //         vesting_start_timestamp: vesting_start_timestamp,
        //         initial_vesting_count: initial_vesting_count,
        //         initial_vesting_consumed: initial_vesting_consumed,
        //         vesting_periodicity: vesting_periodicity,
        //         vesting_count_per_period: vesting_count_per_period,
        //         total_vesting_token_count: total_vesting_token_count,
        //         total_claimed_tokens_till_now: total_claimed_tokens_till_now,
        //         last_claimed_timestamp: last_claimed_timestamp,
        //         tokens_available_to_claim: tokens_available_to_claim,
        //         last_vesting_timestamp: last_vesting_timestamp,
        //         cliff_period: cliff_period,
        //         category_address: Some(category_address.clone()),
        //     },
        // );
        // match vested_amount {
        //     Ok(va) => {
        //         assert_eq!(va.amount, Uint128::zero());
        //     }
        //     Err(e) => {
        //         assert_eq!(1, 0);
        //     }
        // }
    }
}

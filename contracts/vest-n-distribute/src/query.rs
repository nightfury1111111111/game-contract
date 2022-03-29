use cosmwasm_std::{Deps, StdError, StdResult};
use cw20::{BalanceResponse, Cw20QueryMsg};
use crate::state::{CONFIG, VESTING_DETAILS, VestingDetails};

pub fn query_vesting_details(deps: Deps, address: String) -> StdResult<VestingDetails> {
    let address = deps.api.addr_validate(&address)?;
    let vd = VESTING_DETAILS.may_load(deps.storage, &address)?;
    match vd {
        Some(vd) => return Ok(vd),
        None => return Err(StdError::generic_err("No vesting details found")),
    };
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let config = CONFIG.load(deps.storage)?;
    let balance_msg = Cw20QueryMsg::Balance {
        address: address.to_string(),
    };
    let balance_response: cw20::BalanceResponse = deps
        .querier
        .query_wasm_smart(config.fury_token_address.clone(), &balance_msg)?;

    let balance = balance_response.balance;
    Ok(BalanceResponse { balance })
}

// pub fn query_minter(deps: Deps,
//     address: String,
// ) -> StdResult<Option<MinterResponse>> {
//     // let meta = TOKEN_INFO.load(deps.storage)?;
//     let meta = CONFIG.load(deps.storage)?;
//     // let meta = deps.api.addr_validate(&address)?;
//     let minter = match meta.mint {
//         Some(m) => Some(MinterResponse {
//             minter: m.minter.into(),
//             cap: m.cap,
//         }),
//         None => None,
//     };
//     Ok(minter)
// }

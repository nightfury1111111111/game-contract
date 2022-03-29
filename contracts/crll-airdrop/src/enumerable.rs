use cosmwasm_std::{Deps, Order, StdResult};
use cw20::{AllAccountsResponse, AllAllowancesResponse, AllowanceInfo};

use crate::state::{ALLOWANCES};
use cw_storage_plus::Bound;

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

pub fn query_all_allowances(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllAllowancesResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let allowances: StdResult<Vec<AllowanceInfo>> = ALLOWANCES
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok(AllowanceInfo {
                spender: String::from_utf8(k)?,
                allowance: v.allowance,
                expires: v.expires,
            })
        })
        .collect();
    Ok(AllAllowancesResponse {
        allowances: allowances?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, DepsMut, Uint128};
    use cw20::{Cw20Coin, Expiration, TokenInfoResponse};

    use crate::msg::{ExecuteMsg, InstantiateMsg};

}

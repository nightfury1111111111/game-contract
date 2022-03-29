use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Allowance is expired")]
    Expired {},

    #[error("No allowance for this account")]
    NoAllowance {},

    #[error("Logo binary data exceeds 5KB limit")]
    LogoTooBig {},

    #[error("The {functionality} functionality must be called directly")]
    CallExecute {
        functionality: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },

    #[error("Fees received = {received}uusd whereas required = {required}uusd")]
    InsufficientFees {
        received: Uint128,
        required: Uint128,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

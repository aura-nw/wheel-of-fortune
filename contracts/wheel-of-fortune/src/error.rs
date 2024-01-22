use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid address")]
    InvalidAddress {},

    #[error("Insufficent reward")]
    InsufficentReward {},

    #[error("Text too long")]
    TextTooLong {},

    #[error("Wheel is activated")]
    WheelActivated {},

    #[error("Wheel is not activated")]
    WheelNotActivated {},

    #[error("Wheel is not started")]
    WheelNotStarted {},

    #[error("Wheel is ended")]
    WheelEnded {},

    #[error("Wheel is not ended")]
    WheelNotEnded {},

    #[error("Invalid time setting")]
    InvalidTimeSetting {},

    #[error("Invalid randomness")]
    InvalidRandomness {},

    #[error("Player not found")]
    PlayerNotFound {},

    #[error("Invalid number spins")]
    InvalidNumberSpins {},

    #[error("Too many nfts")]
    TooManyNfts {},

    #[error("Too many slots")]
    TooManySlots {},

    #[error("Too many rewards")]
    TooManyRewards {},

    #[error("Insufficent fund")]
    InsufficentFund {},

    #[error("Invalid slot reward")]
    InvalidSlotReward {},

    #[error("Random job not found")]
    RandomJobNotFound {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}

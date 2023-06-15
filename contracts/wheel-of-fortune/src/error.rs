use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

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

    #[error("Too many spins")]
    TooManySpins {},

    #[error("Too many nfts")]
    TooManyNfts {},

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

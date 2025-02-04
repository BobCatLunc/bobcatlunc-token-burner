use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("No LUNA received")]
    NoLunaReceived {},

    #[error("No USTC received")]
    NoUstcReceived {},

    #[error("Unknown reply ID: {id}")]
    UnknownReplyId { id: u64 },
	
	#[error("Invalid tax rate")]
    InvalidTaxRate {},
	
	#[error("Invalid or empty message provided")]
    InvalidMessage {},
}
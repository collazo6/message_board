use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("Please sign up to perform actions on the message board.")]
    NotSignedUp {},

    #[error("Please sign in with your existing account.")]
    AlreadyHaveAccount {},

    #[error("We apologize. It seems that you are not allowed to create a post.")]
    Blacklisted {},

    #[error("We apologize. This post is not available.")]
    PostNotAvailable {},

    #[error("The inputted Address does not correspond to an active user.")]
    UserUnavailable {},
}

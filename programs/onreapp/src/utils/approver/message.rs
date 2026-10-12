use anchor_lang::prelude::Pubkey;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

/// Message structure for approval verification
///
/// This structure contains the data required to verify that a user has received
/// approval from a trusted authority to perform a specific action within the program.
/// The approval is scoped to one concrete offer so a signature cannot be replayed
/// against another approval-gated offer for the same user.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct ApprovalMessage {
    /// The program ID this approval is valid for
    pub program_id: Pubkey,
    /// The offer account this approval is valid for
    pub offer: Pubkey,
    /// The user public key that is approved
    pub user_pubkey: Pubkey,
    /// Unix timestamp when this approval expires
    pub expiry_unix: u64,
}

use anchor_lang::prelude::Pubkey;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};

/// Message structure for approval verification.
///
/// The approval is scoped to one concrete offer so that a valid signature for one
/// approval-gated market cannot be replayed against another market for the same user.
///
/// This serialized message is part of the off-chain approval signing protocol.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct ApprovalMessage {
    /// The program ID this approval is valid for.
    pub program_id: Pubkey,
    /// The offer account this approval is valid for.
    pub offer: Pubkey,
    /// The user public key that is approved.
    pub user_pubkey: Pubkey,
    /// Unix timestamp when this approval expires.
    pub expiry_unix: u64,
}

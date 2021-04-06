//! Sigma protocols

// TODO: remove when sigma protocol is fully implemented
#![allow(dead_code)]

pub mod private_input;
pub mod prover;
pub mod verifier;

mod challenge;
mod dlog_protocol;
mod fiat_shamir;
mod sig_serializer;
mod unchecked_tree;
mod unproven_tree;

use ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use k256::Scalar;

use dlog_protocol::FirstDlogProverMessage;
use std::convert::TryInto;
use unchecked_tree::{UncheckedSigmaTree, UncheckedTree};
use unproven_tree::{UnprovenLeaf, UnprovenSchnorr, UnprovenTree};

use self::challenge::Challenge;
use self::unchecked_tree::UncheckedSchnorr;

extern crate derive_more;
use derive_more::From;

/** The message sent by a prover to its associated verifier as part of a sigma protocol interaction. */
pub(crate) trait ProverMessage {
    /// serialized message
    fn bytes(&self) -> Vec<u8>;
}

/** First message from the prover (message `a` of `SigmaProtocol`)*/
#[derive(PartialEq, Debug, Clone)]
pub enum FirstProverMessage {
    /// Discrete log
    FirstDlogProverMessage(FirstDlogProverMessage),
    /// DH tupl
    FirstDhtProverMessage,
}

impl ProverMessage for FirstProverMessage {
    fn bytes(&self) -> Vec<u8> {
        match self {
            FirstProverMessage::FirstDlogProverMessage(fdpm) => fdpm.bytes(),
            FirstProverMessage::FirstDhtProverMessage => todo!(),
        }
    }
}

/// Proof tree
#[derive(PartialEq, Debug, Clone, From)]
pub(crate) enum ProofTree {
    /// Unchecked tree
    UncheckedTree(UncheckedTree),
    /// Unproven tree
    UnprovenTree(UnprovenTree),
}

impl ProofTree {
    /// Create a new proof tree with a new challenge
    pub(crate) fn with_challenge(&self, challenge: Challenge) -> ProofTree {
        match self {
            ProofTree::UncheckedTree(_) => todo!(),
            ProofTree::UnprovenTree(ut) => match ut {
                UnprovenTree::UnprovenLeaf(ul) => match ul {
                    UnprovenLeaf::UnprovenSchnorr(us) => ProofTree::UnprovenTree(
                        UnprovenSchnorr {
                            challenge_opt: Some(challenge),
                            ..us.clone()
                        }
                        .into(),
                    ),
                },
                UnprovenTree::UnprovenConjecture(_) => todo!(),
            },
        }
    }
}

impl From<UncheckedSchnorr> for ProofTree {
    fn from(v: UncheckedSchnorr) -> Self {
        UncheckedTree::UncheckedSigmaTree(v.into()).into()
    }
}

/// Proof tree leaf
pub(crate) trait ProofTreeLeaf {
    /// Get proposition
    fn proposition(&self) -> SigmaBoolean;

    /// Get commitment
    fn commitment_opt(&self) -> Option<FirstProverMessage>;
}

/** Size of the binary representation of any group element (2 ^ groupSizeBits == <number of elements in a group>) */
pub(crate) const GROUP_SIZE_BITS: usize = 256;
/** Number of bytes to represent any group element as byte array */
pub(crate) const GROUP_SIZE: usize = GROUP_SIZE_BITS / 8;

/// Byte array of Group size (32 bytes)
#[derive(PartialEq, Eq, Debug, Clone)]
pub(crate) struct GroupSizedBytes(pub(crate) Box<[u8; GROUP_SIZE]>);

impl From<GroupSizedBytes> for Scalar {
    fn from(b: GroupSizedBytes) -> Self {
        let sl: &[u8] = b.0.as_ref();
        Scalar::from_bytes_reduced(sl.try_into().expect(""))
    }
}

impl From<&[u8; GROUP_SIZE]> for GroupSizedBytes {
    fn from(b: &[u8; GROUP_SIZE]) -> Self {
        GroupSizedBytes(Box::new(*b))
    }
}

/** A size of challenge in Sigma protocols, in bits.
 * If this anything but 192, threshold won't work, because we have polynomials over GF(2^192) and no others.
 * We get the challenge by reducing hash function output to proper value.
 */
pub const SOUNDNESS_BITS: usize = 192;
/// A size of challenge in Sigma protocols, in bytes
pub const SOUNDNESS_BYTES: usize = SOUNDNESS_BITS / 8;

#[cfg(test)]
#[cfg(feature = "arbitrary")]
mod tests {
    use super::*;

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn ensure_soundness_bits() {
        // see SOUNDNESS_BITS doc comment
        assert!(SOUNDNESS_BITS < GROUP_SIZE_BITS);
        // blake2b hash function requirements
        assert!(SOUNDNESS_BYTES * 8 <= 512);
        assert!(SOUNDNESS_BYTES % 8 == 0);
    }
}

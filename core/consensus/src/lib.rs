#![feature(test)]
#![allow(
    clippy::type_complexity,
    clippy::suspicious_else_formatting,
    clippy::mutable_key_type
)]

pub mod adapter;
pub mod consensus;
pub mod engine;
pub mod message;
pub mod status;
pub mod synchronization;
pub mod types;
pub mod util;
pub mod wal;

use std::error::Error;

use common_crypto::Error as CryptoError;

use protocol::types::{ExitReason, Hash, MerkleRoot};
use protocol::{Display, ProtocolError, ProtocolErrorKind};

pub use crate::adapter::OverlordConsensusAdapter;
pub use crate::consensus::OverlordConsensus;
pub use crate::synchronization::{OverlordSynchronization, RichBlock, SyncStatus, SYNC_STATUS};
pub use crate::wal::{ConsensusWal, SignedTxsWAL};
pub use overlord::{types::Node, DurationConfig};

#[derive(Clone, Debug, Display, PartialEq, Eq)]
pub enum ConsensusType {
    #[display(fmt = "Signed Proposal")]
    SignedProposal,

    #[display(fmt = "Signed Vote")]
    SignedVote,

    #[display(fmt = "Aggregated Vote")]
    AggregateVote,

    #[display(fmt = "Rich Height")]
    RichHeight,

    #[display(fmt = "Rpc Pull Blocks")]
    RpcPullBlocks,

    #[display(fmt = "Rpc Pull Transactions")]
    RpcPullTxs,

    #[display(fmt = "Signed Choke")]
    SignedChoke,

    #[display(fmt = "WAL Signed Transactions")]
    WALSignedTxs,
}

/// Consensus errors defines here.
#[derive(Debug, Display)]
pub enum ConsensusError {
    /// Check block error.
    #[display(fmt = "Check invalid prev_hash, expect {:?} get {:?}", expect, actual)]
    InvalidPrevhash { expect: Hash, actual: Hash },

    #[display(fmt = "Check invalid order root, expect {:?} get {:?}", expect, actual)]
    InvalidOrderRoot {
        expect: MerkleRoot,
        actual: MerkleRoot,
    },

    #[display(fmt = "Check invalid state root, expect {:?} get {:?}", expect, actual)]
    InvalidStateRoot {
        expect: MerkleRoot,
        actual: MerkleRoot,
    },

    #[display(
        fmt = "Check invalid receipts root, expect {:?} get {:?}",
        expect,
        actual
    )]
    InvalidReceiptsRoot {
        expect: MerkleRoot,
        actual: MerkleRoot,
    },

    #[display(
        fmt = "Check invalid order signed transactions hash, expect {:?} get {:?}",
        expect,
        actual
    )]
    InvalidOrderSignedTransactionsHash { expect: Hash, actual: Hash },

    #[display(fmt = "Check invalid status vec")]
    InvalidStatusVec,

    /// Decode consensus message error.
    #[display(fmt = "Decode {:?} message failed", _0)]
    DecodeErr(ConsensusType),

    /// Encode consensus message error.
    #[display(fmt = "Encode {:?} message failed", _0)]
    EncodeErr(ConsensusType),

    /// Overlord consensus protocol error.
    #[display(fmt = "Overlord error {:?}", _0)]
    OverlordErr(Box<dyn Error + Send>),

    /// Consensus missed last block proof.
    #[display(
        fmt = "Invalid proof block number, expect {}, actual {}",
        expect,
        actual
    )]
    InvalidProof { expect: u64, actual: u64 },

    /// Consensus missed the pill.
    #[display(fmt = "Consensus missed pill cooresponding {:?}", _0)]
    MissingPill(Hash),

    /// Invalid timestamp
    #[display(fmt = "Consensus invalid timestamp")]
    InvalidTimestamp,

    /// Consensus missed the block header.
    #[display(fmt = "Consensus missed block header of {} block", _0)]
    MissingBlockHeader(u64),

    /// This boxed error should be a `CryptoError`.
    #[display(fmt = "Crypto error {:?}", _0)]
    CryptoErr(Box<CryptoError>),

    #[display(fmt = "Synchronization {} block error", _0)]
    VerifyTransaction(u64),

    #[display(fmt = "Synchronization/Consensus {} block error : {}", _0, _1)]
    VerifyBlockHeader(u64, BlockHeaderField),

    #[display(fmt = "Synchronization/Consensus {} block error : {}", _0, _1)]
    VerifyProof(u64, BlockProofField),

    ///
    #[display(fmt = "Execute transactions error {:?}", _0)]
    ExecuteErr(String),

    ///
    WALErr(std::io::Error),

    #[display(fmt = "Call EVM error {:?}", _0)]
    CallEvm(ExitReason),

    #[display(fmt = "Storage item not found")]
    StorageItemNotFound,

    #[display(fmt = "Lock in sync")]
    LockInSync,

    #[display(fmt = "Wal transactions mismatch, height {}", _0)]
    WalTxsMismatch(u64),

    #[display(
        fmt = "Commit an outdated block, block_height {}, last_committed_height {}",
        _0,
        _1
    )]
    OutdatedCommit(u64, u64),

    /// Other error used for very few errors.
    #[display(fmt = "{:?}", _0)]
    Other(String),

    #[display(fmt = "{:?}", _0)]
    SystemTime(std::time::SystemTimeError),

    #[display(fmt = "parse file name into timestamp error")]
    FileNameTimestamp,

    #[display(fmt = "consensus wal dir doesn't exist")]
    ConsensusWalDirNotExist,

    #[display(fmt = "no consensus wal file available")]
    ConsensusWalNoWalFile,

    #[display(fmt = "Confused metadata range [{}, {})!", _0, _1)]
    ConfusedMetadata(u64, u64),
}

#[derive(Debug, Display)]
pub enum BlockHeaderField {
    #[display(fmt = "The prev_hash mismatch the previous block")]
    PreviousBlockHash,

    #[display(fmt = "The prev_hash mismatch the hash in the proof field")]
    ProofHash,

    #[display(fmt = "The proposer is not in the committee")]
    Proposer,

    #[display(fmt = "There is at least one validator not in the committee")]
    Validator,

    #[display(fmt = "There is at least one validator's weight mismatch")]
    Weight,
}

#[derive(Debug, Display)]
pub enum BlockProofField {
    #[display(fmt = "The bit_map has error with committer, can't get signed voters")]
    BitMap,

    #[display(fmt = "The proof signature is fraud or error")]
    Signature,

    #[display(fmt = "Heights of block and proof diverse, block {}, proof {}", _0, _1)]
    HeightMismatch(u64, u64),

    #[display(fmt = "Hash of block and proof diverse")]
    HashMismatch,

    #[display(fmt = "There is at least one validator not in the committee")]
    Validator,

    #[display(fmt = "There is at least one validator's weight mismatch")]
    Weight,

    #[display(fmt = "There is at least one validator's weight missing")]
    WeightNotFound,
}

impl Error for ConsensusError {}

impl From<ConsensusError> for ProtocolError {
    fn from(err: ConsensusError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Consensus, Box::new(err))
    }
}

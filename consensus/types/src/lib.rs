//! Ethereum 2.0 types
// Clippy lint set up
#![cfg_attr(
    not(test),
    deny(
        clippy::arithmetic_side_effects,
        clippy::disallowed_methods,
        clippy::indexing_slicing
    )
)]

#[macro_use]
pub mod test_utils;

pub mod aggregate_and_proof;
pub mod application_domain;
pub mod attestation;
pub mod attestation_data;
pub mod attestation_duty;
pub mod attester_slashing;
pub mod beacon_block;
pub mod beacon_block_body;
pub mod beacon_block_header;
pub mod beacon_committee;
pub mod beacon_state;
pub mod bls_to_execution_change;
pub mod builder_bid;
pub mod chain_spec;
pub mod checkpoint;
pub mod consolidation_request;
pub mod consts;
pub mod contribution_and_proof;
pub mod deposit;
pub mod deposit_data;
pub mod deposit_message;
pub mod deposit_request;
pub mod deposit_tree_snapshot;
pub mod dumb_macros;
pub mod enr_fork_id;
pub mod eth1_data;
pub mod eth_spec;
pub mod execution_bid;
pub mod execution_block_hash;
pub mod execution_payload;
pub mod execution_payload_envelope;
pub mod execution_payload_header;
pub mod fork;
pub mod fork_data;
pub mod fork_name;
pub mod fork_versioned_response;
pub mod graffiti;
pub mod historical_batch;
pub mod historical_summary;
pub mod indexed_attestation;
pub mod indexed_payload_attestation;
pub mod light_client_bootstrap;
pub mod light_client_finality_update;
pub mod light_client_optimistic_update;
pub mod light_client_update;
pub mod payload;
pub mod payload_attestation;
pub mod payload_attestation_data;
pub mod payload_attestation_message;
pub mod pending_attestation;
pub mod pending_balance_deposit;
pub mod pending_consolidation;
pub mod pending_partial_withdrawal;
pub mod proposer_preparation_data;
pub mod proposer_slashing;
pub mod relative_epoch;
pub mod selection_proof;
pub mod shuffling_id;
pub mod signed_aggregate_and_proof;
pub mod signed_beacon_block;
pub mod signed_beacon_block_header;
pub mod signed_bls_to_execution_change;
pub mod signed_contribution_and_proof;
pub mod signed_execution_bid;
pub mod signed_execution_payload_envelope;
pub mod signed_voluntary_exit;
pub mod signing_data;
pub mod sync_committee_subscription;
pub mod sync_duty;
pub mod validator;
pub mod validator_subscription;
pub mod voluntary_exit;
pub mod withdrawal_credentials;
pub mod withdrawal_request;
#[macro_use]
pub mod slot_epoch_macros;
pub mod activation_queue;
pub mod config_and_preset;
pub mod execution_block_header;
pub mod fork_context;
pub mod participation_flags;
pub mod preset;
pub mod slot_epoch;
pub mod subnet_id;
pub mod sync_aggregate;
pub mod sync_aggregator_selection_data;
pub mod sync_committee;
pub mod sync_committee_contribution;
pub mod sync_committee_message;
pub mod sync_selection_proof;
pub mod sync_subnet_id;
pub mod validator_registration_data;
pub mod withdrawal;

pub mod epoch_cache;
pub mod slot_data;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub mod blob_sidecar;
pub mod data_column_sidecar;
pub mod data_column_subnet_id;
pub mod light_client_header;
pub mod non_zero_usize;
pub mod runtime_var_list;

pub use crate::activation_queue::ActivationQueue;
pub use crate::aggregate_and_proof::{
    AggregateAndProof, AggregateAndProofBase, AggregateAndProofElectra, AggregateAndProofRef,
};
pub use crate::attestation::{
    Attestation, AttestationBase, AttestationElectra, AttestationRef, AttestationRefMut,
    Error as AttestationError,
};
pub use crate::attestation_data::AttestationData;
pub use crate::attestation_duty::AttestationDuty;
pub use crate::attester_slashing::{
    AttesterSlashing, AttesterSlashingBase, AttesterSlashingElectra, AttesterSlashingOnDisk,
    AttesterSlashingRef, AttesterSlashingRefOnDisk,
};
pub use crate::beacon_block::{
    BeaconBlock, BeaconBlockAltair, BeaconBlockBase, BeaconBlockBellatrix, BeaconBlockCapella,
    BeaconBlockDeneb, BeaconBlockEIP7732, BeaconBlockElectra, BeaconBlockRef, BeaconBlockRefMut,
    BlindedBeaconBlock, BlockImportSource, EmptyBlock,
};
pub use crate::beacon_block_body::{
    BeaconBlockBody, BeaconBlockBodyAltair, BeaconBlockBodyBase, BeaconBlockBodyBellatrix,
    BeaconBlockBodyCapella, BeaconBlockBodyDeneb, BeaconBlockBodyEIP7732, BeaconBlockBodyElectra,
    BeaconBlockBodyRef, BeaconBlockBodyRefMut,
};
pub use crate::beacon_block_header::BeaconBlockHeader;
pub use crate::beacon_committee::{BeaconCommittee, OwnedBeaconCommittee};
pub use crate::beacon_state::{Error as BeaconStateError, *};
pub use crate::blob_sidecar::{BlobIdentifier, BlobSidecar, BlobSidecarList, BlobsList};
pub use crate::bls_to_execution_change::BlsToExecutionChange;
pub use crate::chain_spec::{ChainSpec, Config, Domain};
pub use crate::checkpoint::Checkpoint;
pub use crate::config_and_preset::{
    ConfigAndPreset, ConfigAndPresetCapella, ConfigAndPresetDeneb, ConfigAndPresetElectra,
};
pub use crate::consolidation_request::ConsolidationRequest;
pub use crate::contribution_and_proof::ContributionAndProof;
pub use crate::data_column_sidecar::{
    ColumnIndex, DataColumnIdentifier, DataColumnSidecar, DataColumnSidecarList,
};
pub use crate::data_column_subnet_id::DataColumnSubnetId;
pub use crate::deposit::{Deposit, DEPOSIT_TREE_DEPTH};
pub use crate::deposit_data::DepositData;
pub use crate::deposit_message::DepositMessage;
pub use crate::deposit_request::DepositRequest;
pub use crate::deposit_tree_snapshot::{DepositTreeSnapshot, FinalizedExecutionBlock};
pub use crate::enr_fork_id::EnrForkId;
pub use crate::epoch_cache::{EpochCache, EpochCacheError, EpochCacheKey};
pub use crate::eth1_data::Eth1Data;
pub use crate::eth_spec::EthSpecId;
pub use crate::execution_bid::ExecutionBid;
pub use crate::execution_block_hash::ExecutionBlockHash;
pub use crate::execution_block_header::{EncodableExecutionBlockHeader, ExecutionBlockHeader};
pub use crate::execution_payload::{
    ExecutionPayload, ExecutionPayloadBellatrix, ExecutionPayloadCapella, ExecutionPayloadDeneb,
    ExecutionPayloadEIP7732, ExecutionPayloadElectra, ExecutionPayloadRef, Transaction,
    Transactions, Withdrawals,
};
pub use crate::execution_payload_envelope::{
    ExecutionPayloadEnvelope, ExecutionPayloadEnvelopeEIP7732,
};
pub use crate::execution_payload_header::{
    ExecutionPayloadHeader, ExecutionPayloadHeaderBellatrix, ExecutionPayloadHeaderCapella,
    ExecutionPayloadHeaderDeneb, ExecutionPayloadHeaderElectra, ExecutionPayloadHeaderRef,
    ExecutionPayloadHeaderRefMut,
};
pub use crate::fork::Fork;
pub use crate::fork_context::ForkContext;
pub use crate::fork_data::ForkData;
pub use crate::fork_name::{ForkName, InconsistentFork};
pub use crate::fork_versioned_response::{ForkVersionDeserialize, ForkVersionedResponse};
pub use crate::graffiti::{Graffiti, GRAFFITI_BYTES_LEN};
pub use crate::historical_batch::HistoricalBatch;
pub use crate::indexed_attestation::{
    IndexedAttestation, IndexedAttestationBase, IndexedAttestationElectra, IndexedAttestationRef,
};
pub use crate::indexed_payload_attestation::IndexedPayloadAttestation;
pub use crate::light_client_bootstrap::{
    LightClientBootstrap, LightClientBootstrapAltair, LightClientBootstrapCapella,
    LightClientBootstrapDeneb, LightClientBootstrapElectra,
};
pub use crate::light_client_finality_update::{
    LightClientFinalityUpdate, LightClientFinalityUpdateAltair, LightClientFinalityUpdateCapella,
    LightClientFinalityUpdateDeneb, LightClientFinalityUpdateElectra,
};
pub use crate::light_client_header::{
    LightClientHeader, LightClientHeaderAltair, LightClientHeaderCapella, LightClientHeaderDeneb,
    LightClientHeaderElectra,
};
pub use crate::light_client_optimistic_update::{
    LightClientOptimisticUpdate, LightClientOptimisticUpdateAltair,
    LightClientOptimisticUpdateCapella, LightClientOptimisticUpdateDeneb,
    LightClientOptimisticUpdateElectra,
};
pub use crate::light_client_update::{
    Error as LightClientUpdateError, LightClientUpdate, LightClientUpdateAltair,
    LightClientUpdateCapella, LightClientUpdateDeneb, LightClientUpdateElectra,
};
pub use crate::participation_flags::ParticipationFlags;
pub use crate::payload::{
    AbstractExecPayload, BlindedPayload, BlindedPayloadBellatrix, BlindedPayloadCapella,
    BlindedPayloadDeneb, BlindedPayloadElectra, BlindedPayloadRef, BlockType, ExecPayload,
    FullPayload, FullPayloadBellatrix, FullPayloadCapella, FullPayloadDeneb, FullPayloadElectra,
    FullPayloadRef, OwnedExecPayload,
};
pub use crate::payload_attestation::PayloadAttestation;
pub use crate::payload_attestation_data::{PayloadAttestationData, PayloadStatus};
pub use crate::payload_attestation_message::PayloadAttestationMessage;
pub use crate::pending_attestation::PendingAttestation;
pub use crate::pending_balance_deposit::PendingBalanceDeposit;
pub use crate::pending_consolidation::PendingConsolidation;
pub use crate::pending_partial_withdrawal::PendingPartialWithdrawal;
pub use crate::preset::{
    AltairPreset, BasePreset, BellatrixPreset, CapellaPreset, DenebPreset, ElectraPreset,
};
pub use crate::proposer_preparation_data::ProposerPreparationData;
pub use crate::proposer_slashing::ProposerSlashing;
pub use crate::relative_epoch::{Error as RelativeEpochError, RelativeEpoch};
pub use crate::runtime_var_list::RuntimeVariableList;
pub use crate::selection_proof::SelectionProof;
pub use crate::shuffling_id::AttestationShufflingId;
pub use crate::signed_aggregate_and_proof::{
    SignedAggregateAndProof, SignedAggregateAndProofBase, SignedAggregateAndProofElectra,
};
pub use crate::signed_beacon_block::{
    ssz_tagged_signed_beacon_block, ssz_tagged_signed_beacon_block_arc, SignedBeaconBlock,
    SignedBeaconBlockAltair, SignedBeaconBlockBase, SignedBeaconBlockBellatrix,
    SignedBeaconBlockCapella, SignedBeaconBlockDeneb, SignedBeaconBlockEIP7732,
    SignedBeaconBlockElectra, SignedBeaconBlockHash, SignedBlindedBeaconBlock,
};
pub use crate::signed_beacon_block_header::SignedBeaconBlockHeader;
pub use crate::signed_bls_to_execution_change::SignedBlsToExecutionChange;
pub use crate::signed_contribution_and_proof::SignedContributionAndProof;
pub use crate::signed_execution_bid::SignedExecutionBid;
pub use crate::signed_execution_payload_envelope::SignedExecutionPayloadEnvelope;
pub use crate::signed_voluntary_exit::SignedVoluntaryExit;
pub use crate::signing_data::{SignedRoot, SigningData};
pub use crate::slot_epoch::{Epoch, Slot};
pub use crate::subnet_id::SubnetId;
pub use crate::sync_aggregate::SyncAggregate;
pub use crate::sync_aggregator_selection_data::SyncAggregatorSelectionData;
pub use crate::sync_committee::SyncCommittee;
pub use crate::sync_committee_contribution::{SyncCommitteeContribution, SyncContributionData};
pub use crate::sync_committee_message::SyncCommitteeMessage;
pub use crate::sync_committee_subscription::SyncCommitteeSubscription;
pub use crate::sync_duty::SyncDuty;
pub use crate::sync_selection_proof::SyncSelectionProof;
pub use crate::sync_subnet_id::SyncSubnetId;
pub use crate::validator::Validator;
pub use crate::validator_registration_data::*;
pub use crate::validator_subscription::ValidatorSubscription;
pub use crate::voluntary_exit::VoluntaryExit;
pub use crate::withdrawal::Withdrawal;
pub use crate::withdrawal_credentials::WithdrawalCredentials;
pub use crate::withdrawal_request::WithdrawalRequest;
pub use fixed_bytes::FixedBytesExtended;

pub type CommitteeIndex = u64;
pub type Hash256 = fixed_bytes::Hash256;
pub type Uint256 = fixed_bytes::Uint256;
pub type Address = fixed_bytes::Address;
pub type ForkVersion = [u8; 4];
pub type BLSFieldElement = Uint256;
pub type Blob<E> = FixedVector<u8, <E as EthSpec>::BytesPerBlob>;
pub type KzgProofs<E> = VariableList<KzgProof, <E as EthSpec>::MaxBlobCommitmentsPerBlock>;
pub type VersionedHash = Hash256;
pub type Hash64 = alloy_primitives::B64;

pub use bls::{
    AggregatePublicKey, AggregateSignature, Keypair, PublicKey, PublicKeyBytes, SecretKey,
    Signature, SignatureBytes,
};
pub use kzg::{KzgCommitment, KzgProof, VERSIONED_HASH_VERSION_KZG};
pub use milhouse::{self, List, Vector};
pub use ssz_types::{typenum, typenum::Unsigned, BitList, BitVector, FixedVector, VariableList};
pub use superstruct::superstruct;

use alloy_primitives::B256;
use helios_consensus_core::consensus_spec::MainnetConsensusSpec;
use helios_consensus_core::types::Forks;
use helios_consensus_core::types::{FinalityUpdate, LightClientStore, Update};
use serde::{Deserialize, Serialize};

/// Input data structure for the light client verification
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProofInputs {
    pub sync_committee_updates: Vec<Update<MainnetConsensusSpec>>,
    pub finality_update: FinalityUpdate<MainnetConsensusSpec>,
    pub expected_current_slot: u64,
    pub store: LightClientStore<MainnetConsensusSpec>,
    pub genesis_root: B256,
    pub forks: Forks,
}

/// Output data structure with verification results
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProofOutputs {
    pub success: bool,
    pub new_header: B256,
    pub new_head: u64,
    pub sync_committee_hash: B256,
    pub next_sync_committee_hash: B256,
}

/// State data captured before and after verification
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VerificationState {
    pub prev_header: B256,
    pub prev_head: u64,
    pub outputs: ProofOutputs,
}

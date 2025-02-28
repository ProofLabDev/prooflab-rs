use alloy_primitives::B256;
use helios_consensus_core::{
    apply_finality_update, apply_update, verify_finality_update, verify_update,
};
use helios_light_client_lib::{ProofInputs, ProofOutputs, VerificationState};
use prooflab_io;
use tree_hash::TreeHash;

fn main() {
    // Read inputs
    let encoded_inputs: Vec<u8> = prooflab_io::read();

    let ProofInputs {
        sync_committee_updates,
        finality_update,
        expected_current_slot,
        mut store,
        genesis_root,
        forks,
    } = serde_cbor::from_slice(&encoded_inputs).unwrap();

    let prev_header: B256 = store.finalized_header.beacon().tree_hash_root();
    let prev_head = store.finalized_header.beacon().slot;

    // Process sync committee updates
    let mut success = true;
    for (index, update) in sync_committee_updates.iter().enumerate() {
        let update_is_valid =
            verify_update(update, expected_current_slot, &store, genesis_root, &forks).is_ok();

        if !update_is_valid {
            println!("Update {} is invalid!", index + 1);
            success = false;
            break;
        }

        println!("Update {} is valid.", index + 1);
        apply_update(&mut store, update);
    }

    // Only apply finality update if sync committee updates were successful
    if success {
        let finality_update_is_valid = verify_finality_update(
            &finality_update,
            expected_current_slot,
            &store,
            genesis_root,
            &forks,
        )
        .is_ok();

        if !finality_update_is_valid {
            println!("Finality update is invalid!");
            success = false;
        } else {
            println!("Finality update is valid.");
            apply_finality_update(&mut store, &finality_update);
        }
    }

    // Get final state values
    let new_header: B256 = store.finalized_header.beacon().tree_hash_root();
    let sync_committee_hash: B256 = store.current_sync_committee.tree_hash_root();
    let next_sync_committee_hash: B256 = match &store.next_sync_committee {
        Some(next_sync_committee) => next_sync_committee.tree_hash_root(),
        None => B256::ZERO,
    };
    let new_head = store.finalized_header.beacon().slot;

    // Prepare outputs
    let outputs = ProofOutputs {
        success,
        new_header,
        new_head,
        sync_committee_hash,
        next_sync_committee_hash,
    };

    // Return results for the verification state
    prooflab_io::commit(&VerificationState {
        prev_header,
        prev_head,
        outputs,
    });
}

fn input() {
    let input = include_bytes!("../proof_inputs.cbor");
    prooflab_io::write(&input.to_vec());
}

fn output() {
    let state: VerificationState = prooflab_io::out();

    println!("=== Helios Light Client Verification ===");
    println!("Previous header: {:?}", state.prev_header);
    println!("Previous head slot: {}", state.prev_head);
    println!("Verification successful: {}", state.outputs.success);

    if state.outputs.success {
        println!("\nNew header: {:?}", state.outputs.new_header);
        println!("New head slot: {}", state.outputs.new_head);
        println!(
            "Sync committee hash: {:?}",
            state.outputs.sync_committee_hash
        );
        println!(
            "Next sync committee hash: {:?}",
            state.outputs.next_sync_committee_hash
        );
        println!("\nSuccessfully verified and updated light client state inside zkVM!");
    } else {
        println!("\nLight client verification failed!");
    }
}

use aligned_sdk::core::types::Network;
use prooflab::NetworkArg;

#[test]
fn test_network_arg_conversion() {
    // Test conversion from NetworkArg to Network
    let devnet_arg = NetworkArg::Devnet;
    let holesky_arg = NetworkArg::Holesky;
    let holesky_stage_arg = NetworkArg::HoleskyStage;
    
    // Convert to Network
    let devnet: Network = devnet_arg.into();
    let holesky: Network = holesky_arg.into();
    let holesky_stage: Network = holesky_stage_arg.into();
    
    // Verify conversion
    match devnet {
        Network::Devnet => {} // This is expected
        _ => panic!("Expected Network::Devnet"),
    }
    
    match holesky {
        Network::Holesky => {} // This is expected
        _ => panic!("Expected Network::Holesky"),
    }
    
    match holesky_stage {
        Network::HoleskyStage => {} // This is expected
        _ => panic!("Expected Network::HoleskyStage"),
    }
}
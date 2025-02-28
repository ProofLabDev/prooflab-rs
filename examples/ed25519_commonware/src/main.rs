use commonware_cryptography::{Ed25519, Scheme};
use prooflab_io;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() {
    // Read inputs from host
    let (message, namespace, public_key_bytes, signature_bytes): (
        Vec<u8>,
        Option<Vec<u8>>,
        Vec<u8>,
        Vec<u8>,
    ) = prooflab_io::read();

    // Convert namespace to reference format expected by verify
    let namespace_ref = namespace.as_ref().map(|ns| ns.as_slice());

    // Convert raw bytes to the expected types
    let public_key = match commonware_cryptography::ed25519::PublicKey::try_from(&public_key_bytes[..]) {
        Ok(pk) => pk,
        Err(_) => {
            prooflab_io::commit(&message);
            prooflab_io::commit(&false); // Verification failed - invalid public key
            return;
        }
    };

    let signature = match commonware_cryptography::ed25519::Signature::try_from(&signature_bytes[..]) {
        Ok(sig) => sig,
        Err(_) => {
            prooflab_io::commit(&message);
            prooflab_io::commit(&false); // Verification failed - invalid signature
            return;
        }
    };

    // Start verification
    let result = Ed25519::verify(namespace_ref, &message, &public_key, &signature);

    // Return results
    prooflab_io::commit(&message);
    prooflab_io::commit(&result);
}

fn input() {
    // Use deterministic RNG for reproducibility
    let seed = 12345;
    let mut rng = StdRng::seed_from_u64(seed);

    // Create test message
    let message = b"This is a test message for Ed25519 signature verification".to_vec();

    // Optional namespace
    let namespace = Some(b"benchmark".to_vec());

    // Create a new signer
    let mut signer = Ed25519::new(&mut rng);

    // Sign the message
    let namespace_ref = namespace.as_ref().map(|ns| ns.as_slice());
    let signature = signer.sign(namespace_ref, &message);

    // Send to zkVM
    prooflab_io::write(&(
        message,
        namespace,
        signer.public_key().to_vec(),
        signature.to_vec(),
    ));
}

fn output() {
    let (message, is_valid): (Vec<u8>, bool) = prooflab_io::out();

    println!("=== Ed25519 Signature Verification ===");
    println!("Message: {}", String::from_utf8_lossy(&message));
    println!("Signature valid: {}", is_valid);

    if is_valid {
        println!("\nSuccessfully verified Ed25519 signature inside zkVM!");
    } else {
        println!("\nSignature verification failed!");
    }
}
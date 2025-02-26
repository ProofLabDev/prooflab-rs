use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

fn main() {
    let times: u8 = prooflab_io::read();
    println!("Reading {} signatures", times);

    for i in 0..times {
        println!("Verifying signature {}", i);
        let verifying_key_bytes: [u8; 32] = prooflab_io::read();
        let signer =
            VerifyingKey::from_bytes(&verifying_key_bytes).expect("Invalid verifying key bytes");
        let message: Vec<u8> = prooflab_io::read();
        let signature_bytes: Vec<u8> = prooflab_io::read();
        let signature = Signature::from_slice(&signature_bytes).expect("Invalid signature bytes");

        signer
            .verify(&message, &signature)
            .expect("Ed25519 signature verification failed");
        prooflab_io::commit(&(verifying_key_bytes, message));
    }
}

fn input() {
    let times: u8 = 2; // Reduced for testing
    prooflab_io::write(&times);

    for i in 0..times {
        println!("Generating signature {}", i);
        let signing_key = SigningKey::generate(&mut OsRng);
        let message = b"Hello, world!";
        let signature = signing_key.sign(message);

        let verifying_key_bytes = signing_key.verifying_key().to_bytes();
        prooflab_io::write(&verifying_key_bytes);
        prooflab_io::write(&message.to_vec());
        prooflab_io::write(&signature.to_vec());
    }
}

fn output() {
    let (verifying_key_bytes, message): ([u8; 32], Vec<u8>) = prooflab_io::out();
    let signer =
        VerifyingKey::from_bytes(&verifying_key_bytes).expect("Invalid verifying key bytes");

    println!(
        "Verified the signature over message {:?} with key {:?}",
        std::str::from_utf8(&message[..]).unwrap(),
        signer
    );
}

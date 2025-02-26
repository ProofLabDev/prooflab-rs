use k256::{
    ecdsa::{
        signature::{Signer, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    EncodedPoint,
};
use rand_core::OsRng;

fn main() {
    let encoded_verifying_key: EncodedPoint = prooflab_io::read();
    let message: Vec<u8> = prooflab_io::read();
    let signature_der: Vec<u8> = prooflab_io::read();
    let signature = Signature::from_der(&signature_der).unwrap();

    let verifying_key = VerifyingKey::from_encoded_point(&encoded_verifying_key).unwrap();

    // Verify the signature, panicking if verification fails.
    verifying_key
        .verify(&message, &signature)
        .expect("ECDSA signature verification failed");

    prooflab_io::commit(&(encoded_verifying_key, message));
}

fn input() {
    let signing_key = SigningKey::random(&mut OsRng);
    let message = b"This is a message that will be signed, and verified within the zkVM".to_vec();
    let signature: Signature = signing_key.sign(&message);
    let vk = signing_key.verifying_key().to_encoded_point(true);
    prooflab_io::write(&vk);
    prooflab_io::write(&message);
    prooflab_io::write(&signature.to_der().as_bytes().to_vec());
}

fn output() {
    let (receipt_verifying_key, receipt_message): (EncodedPoint, Vec<u8>) = prooflab_io::out();

    println!(
        "Verified the signature over message {:?} with key {}",
        std::str::from_utf8(&receipt_message[..]).unwrap(),
        receipt_verifying_key,
    );
}

use super::ChainSignatureVerifier;
use crate::resources::wallets::traits::WalletError;

#[derive(Debug, Clone)]
pub struct SolanaSignatureVerifier;

impl SolanaSignatureVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SolanaSignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainSignatureVerifier for SolanaSignatureVerifier {
    fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        wallet_address: &str,
    ) -> Result<bool, WalletError> {
        // TODO: Implement proper Ed25519 signature verification
        // This would involve:
        // 1. Convert message to bytes
        // 2. Parse signature from array format (comma-separated bytes or base58)
        // 3. Derive public key from wallet address (base58 decode)
        // 4. Verify using ed25519 cryptography
        //
        // For production, use ed25519-dalek or solana-sdk crate:
        // let pubkey = Pubkey::from_str(wallet_address)?;
        // let sig = Signature::from_str(signature)?;
        // pubkey.verify(message.as_bytes(), &sig)

        println!("Solana signature verification:");
        println!("  Message length: {} chars", message.len());
        println!(
            "  Signature preview: {}...",
            &signature[..signature.len().min(30)]
        );
        println!("  Expected Address: {}", wallet_address);

        // Validate signature is present and has reasonable length
        // Solana signatures come as comma-separated byte arrays from frontend
        // e.g., "1,2,3,4,5..." (64 bytes = ~190 chars with commas)
        if signature.is_empty() {
            return Err(WalletError::InvalidSignature(
                "Solana signature is empty".to_string(),
            ));
        }

        // Check if it's a byte array format (comma-separated numbers)
        let is_byte_array = signature.contains(',')
            && signature
                .split(',')
                .all(|s| s.trim().parse::<u8>().is_ok());

        // Or base58/base64 encoded
        let is_encoded = !signature.contains(',') && signature.len() >= 64;

        if !is_byte_array && !is_encoded {
            return Err(WalletError::InvalidSignature(
                "Invalid Solana signature format".to_string(),
            ));
        }

        // Validate wallet address format
        if !self.validate_address(wallet_address) {
            return Err(WalletError::InvalidAddress(format!(
                "Invalid Solana address: {}",
                wallet_address
            )));
        }

        // Placeholder: Accept valid format signatures
        // In production, implement actual Ed25519 verification
        println!("  [PLACEHOLDER] Solana signature format valid - accepting");
        Ok(true)
    }

    fn validate_address(&self, address: &str) -> bool {
        // Solana addresses are Base58 encoded, typically 32-44 characters
        // Valid Base58 alphabet (no 0, O, I, l)
        const BASE58_ALPHABET: &str =
            "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

        address.len() >= 32
            && address.len() <= 44
            && address.chars().all(|c| BASE58_ALPHABET.contains(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_solana_address() {
        let verifier = SolanaSignatureVerifier::new();

        // Valid addresses (example Solana public keys)
        assert!(verifier.validate_address("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6"));
        assert!(verifier.validate_address("11111111111111111111111111111111")); // System program
        assert!(verifier.validate_address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")); // Token program

        // Invalid addresses
        assert!(!verifier.validate_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e")); // EVM format
        assert!(!verifier.validate_address("short")); // Too short
        assert!(!verifier.validate_address("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM60")); // Contains 0
        assert!(!verifier.validate_address("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJMO")); // Contains O
    }

    #[test]
    fn test_signature_format_validation() {
        let verifier = SolanaSignatureVerifier::new();
        let message = "test message";
        let address = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM6";

        // Valid: byte array format
        let byte_array_sig = (0..64).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
        assert!(verifier
            .verify_signature(message, &byte_array_sig, address)
            .is_ok());

        // Valid: base58 encoded format (64+ chars)
        let encoded_sig = "a".repeat(88); // Typical base58 signature length
        assert!(verifier
            .verify_signature(message, &encoded_sig, address)
            .is_ok());

        // Invalid: empty signature
        assert!(verifier.verify_signature(message, "", address).is_err());
    }
}

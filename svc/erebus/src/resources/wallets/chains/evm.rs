use super::ChainSignatureVerifier;
use crate::resources::wallets::traits::WalletError;

#[derive(Debug, Clone)]
pub struct EvmSignatureVerifier;

impl EvmSignatureVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EvmSignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainSignatureVerifier for EvmSignatureVerifier {
    fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        wallet_address: &str,
    ) -> Result<bool, WalletError> {
        // TODO: Implement proper ECDSA signature verification
        // This would involve:
        // 1. Hash the message with Ethereum's \x19Ethereum Signed Message:\n prefix
        // 2. Recover the public key from signature using secp256k1
        // 3. Derive address from public key (keccak256 hash, take last 20 bytes)
        // 4. Compare with expected address (case-insensitive)
        //
        // For production, use ethers-rs or alloy crate:
        // let recovered = signature.recover(message)?;
        // Ok(recovered == wallet_address)

        println!("EVM signature verification:");
        println!("  Message length: {} chars", message.len());
        println!("  Signature: {}...", &signature[..signature.len().min(20)]);
        println!("  Expected Address: {}", wallet_address);

        // Validate signature format
        if !signature.starts_with("0x") {
            return Err(WalletError::InvalidSignature(
                "EVM signature must start with 0x".to_string(),
            ));
        }

        if signature.len() < 132 {
            return Err(WalletError::InvalidSignature(format!(
                "EVM signature too short: {} chars (expected 132+)",
                signature.len()
            )));
        }

        // Validate signature is valid hex
        if !signature[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(WalletError::InvalidSignature(
                "EVM signature contains invalid hex characters".to_string(),
            ));
        }

        // Placeholder: Accept valid format signatures
        // In production, implement actual ECDSA recovery
        println!("  [PLACEHOLDER] EVM signature format valid - accepting");
        Ok(true)
    }

    fn validate_address(&self, address: &str) -> bool {
        // EVM address format: 0x followed by 40 hex characters
        address.starts_with("0x")
            && address.len() == 42
            && address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_evm_address() {
        let verifier = EvmSignatureVerifier::new();

        // Valid addresses
        assert!(verifier.validate_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44e"));
        assert!(verifier.validate_address("0x0000000000000000000000000000000000000000"));
        assert!(verifier.validate_address("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"));

        // Invalid addresses
        assert!(!verifier.validate_address("742d35Cc6634C0532925a3b844Bc454e4438f44e")); // Missing 0x
        assert!(!verifier.validate_address("0x742d35Cc6634C0532925a3b844Bc454e4438f4")); // Too short
        assert!(!verifier.validate_address("0x742d35Cc6634C0532925a3b844Bc454e4438f44eFF")); // Too long
        assert!(!verifier.validate_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"));
        // Invalid hex
    }

    #[test]
    fn test_signature_format_validation() {
        let verifier = EvmSignatureVerifier::new();
        let message = "test message";
        let address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";

        // Valid signature format (132 chars = 0x + 130 hex)
        let valid_sig = format!("0x{}", "a".repeat(130));
        assert!(verifier
            .verify_signature(message, &valid_sig, address)
            .is_ok());

        // Invalid: missing 0x prefix
        let no_prefix = "a".repeat(130);
        assert!(verifier
            .verify_signature(message, &no_prefix, address)
            .is_err());

        // Invalid: too short
        let short_sig = format!("0x{}", "a".repeat(50));
        assert!(verifier
            .verify_signature(message, &short_sig, address)
            .is_err());
    }
}

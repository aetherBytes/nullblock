use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use sqlx::postgres::PgPoolOptions;
use std::env;

struct EncryptedData {
    ciphertext: Vec<u8>,
    iv: Vec<u8>,
    tag: Vec<u8>,
}

fn encrypt(cipher: &Aes256Gcm, plaintext: &str) -> Result<EncryptedData, String> {
    let mut iv = vec![0u8; 12];
    OsRng.fill_bytes(&mut iv);

    let nonce = Nonce::from_slice(&iv);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let tag = ciphertext[ciphertext.len() - 16..].to_vec();
    let ciphertext_only = ciphertext[..ciphertext.len() - 16].to_vec();

    Ok(EncryptedData {
        ciphertext: ciphertext_only,
        iv,
        tag,
    })
}

fn extract_prefix_suffix(api_key: &str) -> (String, String) {
    let prefix_len = 10.min(api_key.len() / 3);
    let suffix_len = 4.min(api_key.len() / 4);

    let prefix = api_key[..prefix_len].to_string();
    let suffix = if api_key.len() >= suffix_len {
        api_key[api_key.len() - suffix_len..].to_string()
    } else {
        String::new()
    };

    (prefix, suffix)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Agent API Key Seeder ===\n");

    // Load .env.dev if available
    if let Ok(contents) = std::fs::read_to_string(".env.dev") {
        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                if !key.is_empty() && !key.starts_with('#') {
                    env::set_var(key, value);
                }
            }
        }
        println!("Loaded .env.dev");
    } else if let Ok(contents) = std::fs::read_to_string("../../.env.dev") {
        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                if !key.is_empty() && !key.starts_with('#') {
                    env::set_var(key, value);
                }
            }
        }
        println!("Loaded ../../.env.dev");
    }

    // Get environment variables
    let encryption_key = env::var("ENCRYPTION_MASTER_KEY")
        .expect("ENCRYPTION_MASTER_KEY must be set in environment");

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus".to_string());

    println!("Database URL: {}", database_url.replace("REDACTED_DB_PASS", "***"));

    // Verify encryption key format
    let key_bytes = hex::decode(&encryption_key)
        .map_err(|e| format!("Invalid ENCRYPTION_MASTER_KEY hex: {}", e))?;

    if key_bytes.len() != 32 {
        return Err(format!(
            "ENCRYPTION_MASTER_KEY must be 32 bytes (64 hex chars), got {} bytes",
            key_bytes.len()
        ).into());
    }

    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    println!("Encryption key validated (32 bytes AES-256-GCM)\n");

    // Connect to database
    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    println!("Connected to database.\n");

    // Agent keys to seed (OpenRouter only for now)
    let agent_keys = vec![
        ("hecate", "openrouter", "REDACTED_OPENROUTER_KEY_1", "HECATE Primary Key"),
        ("siren", "openrouter", "REDACTED_OPENROUTER_KEY_2", "Siren Primary Key"),
    ];

    for (agent_name, provider, api_key, key_name) in agent_keys {
        println!("Seeding {} / {} ...", agent_name, provider);

        // Encrypt the API key
        let encrypted = encrypt(&cipher, api_key)?;
        let (key_prefix, key_suffix) = extract_prefix_suffix(api_key);

        println!("  Key prefix: {}...", key_prefix);
        println!("  Key suffix: ...{}", key_suffix);

        // Upsert into database
        let result = sqlx::query(
            r#"
            INSERT INTO agent_api_keys
            (agent_name, provider, encrypted_key, encryption_iv, encryption_tag, key_prefix, key_suffix, key_name)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (agent_name, provider)
            DO UPDATE SET
                encrypted_key = EXCLUDED.encrypted_key,
                encryption_iv = EXCLUDED.encryption_iv,
                encryption_tag = EXCLUDED.encryption_tag,
                key_prefix = EXCLUDED.key_prefix,
                key_suffix = EXCLUDED.key_suffix,
                key_name = EXCLUDED.key_name,
                is_active = true,
                updated_at = NOW()
            "#
        )
        .bind(agent_name)
        .bind(provider)
        .bind(&encrypted.ciphertext)
        .bind(&encrypted.iv)
        .bind(&encrypted.tag)
        .bind(&key_prefix)
        .bind(&key_suffix)
        .bind(key_name)
        .execute(&pool)
        .await;

        match result {
            Ok(_) => println!("  Seeded successfully.\n"),
            Err(e) => println!("  ERROR: {}\n", e),
        }
    }

    // Verify seeded keys
    println!("=== Verification ===\n");

    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, bool)>(
        "SELECT agent_name, provider, key_prefix, key_suffix, is_active FROM agent_api_keys ORDER BY agent_name"
    )
    .fetch_all(&pool)
    .await?;

    for (agent_name, provider, prefix, suffix, is_active) in rows {
        let status = if is_active { "ACTIVE" } else { "INACTIVE" };
        println!(
            "  {} / {}: {}...{} [{}]",
            agent_name,
            provider,
            prefix.unwrap_or_default(),
            suffix.unwrap_or_default(),
            status
        );
    }

    println!("\nDone.");
    Ok(())
}

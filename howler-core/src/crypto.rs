use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Simple XOR obfuscation + base64 encoding.
/// This is NOT encryption — it prevents casual reading of the token.
/// Anyone with the source code can reverse it.
const XOR_KEY: &[u8] = b"HowlerWolf2024";

fn xor_obfuscate(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ XOR_KEY[i % XOR_KEY.len()])
        .collect()
}

/// Obfuscate a plaintext token (XOR + base64)
pub fn obfuscate(plaintext: &str) -> String {
    let xored = xor_obfuscate(plaintext.as_bytes());
    STANDARD.encode(&xored)
}

/// Deobfuscate an obfuscated token (base64 + XOR)
pub fn deobfuscate(obfuscated: &str) -> Result<String> {
    let decoded = STANDARD
        .decode(obfuscated)
        .context("Failed to base64-decode token")?;
    let xored = xor_obfuscate(&decoded);
    String::from_utf8(xored).context("Failed to convert deobfuscated token to UTF-8")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretsFile {
    pub iucn: Option<IucnSecrets>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IucnSecrets {
    pub token: String,
}

/// Load obfuscated token from secrets file
pub fn load_iucn_token_from_secrets(secrets_path: &Path) -> Result<Option<String>> {
    if !secrets_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(secrets_path)
        .with_context(|| format!("Failed to read {}", secrets_path.display()))?;

    let secrets: SecretsFile = toml::from_str(&content).context("Failed to parse secrets.toml")?;

    if let Some(iucn) = secrets.iucn {
        let token = deobfuscate(&iucn.token)?;
        Ok(Some(token))
    } else {
        Ok(None)
    }
}

/// Save obfuscated token to secrets file
pub fn save_iucn_token_to_secrets(secrets_path: &Path, token: &str) -> Result<()> {
    let obfuscated = obfuscate(token);

    let secrets = SecretsFile {
        iucn: Some(IucnSecrets { token: obfuscated }),
    };

    let content = toml::to_string_pretty(&secrets).context("Failed to serialize secrets")?;

    // Create parent directory if needed
    if let Some(parent) = secrets_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(secrets_path, content)
        .with_context(|| format!("Failed to write {}", secrets_path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscate_deobfuscate_roundtrip() {
        let original = "my_secret_iucn_token_12345";
        let obfuscated = obfuscate(original);
        let deobfuscated = deobfuscate(&obfuscated).unwrap();
        assert_eq!(original, deobfuscated);
    }

    #[test]
    fn test_obfuscate_produces_different_output() {
        let original = "test_token";
        let obfuscated = obfuscate(original);
        assert_ne!(original, obfuscated);
    }

    #[test]
    fn test_deobfuscate_invalid_base64() {
        let result = deobfuscate("not-valid-base64!!!");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_secrets() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.toml");

        save_iucn_token_to_secrets(&path, "test_iucn_token").unwrap();
        let loaded = load_iucn_token_from_secrets(&path).unwrap();
        assert_eq!(loaded, Some("test_iucn_token".to_string()));
    }
}

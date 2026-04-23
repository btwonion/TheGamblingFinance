//! Argon2id password hashing + verification.
//!
//! Hashes are serialized as PHC strings (`$argon2id$v=19$m=...$...$...`)
//! so the algorithm params live with the hash; this lets us bump cost
//! parameters later without a data migration. We use `Argon2::default()`
//! which is `Argon2id` v19 with the `OWASP 2023` recommended parameters
//! at the time of the `argon2 = "0.5"` release.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash `pw` with Argon2id and a fresh 16-byte random salt. Returns the
/// PHC-encoded string that should be persisted in `users.password_hash`.
pub fn hash_password(pw: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon = Argon2::default();
    let phc = argon.hash_password(pw.as_bytes(), &salt)?;
    Ok(phc.to_string())
}

/// Verify `pw` against a stored PHC string. Returns `false` for any
/// failure mode — including malformed PHC strings — so that callers
/// cannot distinguish "wrong password" from "corrupted hash row" via
/// error text (avoids user enumeration + information leakage).
pub fn verify_password(pw: &str, phc: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(phc) else {
        return false;
    };
    Argon2::default()
        .verify_password(pw.as_bytes(), &parsed)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_roundtrip_accepts_correct_password() {
        let phc = hash_password("hunter2-correct").expect("hash");
        assert!(phc.starts_with("$argon2id$"));
        assert!(verify_password("hunter2-correct", &phc));
    }

    #[test]
    fn verify_rejects_wrong_password() {
        let phc = hash_password("right-answer").expect("hash");
        assert!(!verify_password("wrong-answer", &phc));
    }

    #[test]
    fn verify_rejects_malformed_phc() {
        assert!(!verify_password("whatever", "not-a-phc-string"));
        assert!(!verify_password("whatever", ""));
    }

    #[test]
    fn hash_produces_different_outputs_for_same_input() {
        // Different salts → different PHC strings (non-deterministic).
        let a = hash_password("same").unwrap();
        let b = hash_password("same").unwrap();
        assert_ne!(a, b);
        assert!(verify_password("same", &a));
        assert!(verify_password("same", &b));
    }
}

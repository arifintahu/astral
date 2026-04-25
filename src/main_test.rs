use super::*;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::Mac;

// --- Session Token Tests ---

#[test]
fn test_token_format_is_timestamp_dot_nonce_dot_hmac() {
    let token = create_session_token("secret", "admin").unwrap();
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    assert_eq!(parts.len(), 3, "Token should be timestamp.nonce.hmac");
    assert!(parts[0].parse::<u64>().is_ok(), "First part should be a u64 timestamp");
    assert_eq!(parts[1].len(), 32, "Nonce should be 16 bytes = 32 hex chars");
    assert_eq!(parts[2].len(), 64, "HMAC-SHA256 hex should be 64 chars");
}

#[test]
fn test_verify_accepts_valid_token() {
    let secret = "test_secret";
    let token = create_session_token(secret, "admin").unwrap();
    assert!(verify_session_token(secret, "admin", &token));
}

#[test]
fn test_verify_rejects_wrong_secret() {
    let token = create_session_token("secret_a", "admin").unwrap();
    assert!(!verify_session_token("secret_b", "admin", &token));
}

#[test]
fn test_verify_rejects_wrong_username() {
    let secret = "test_secret";
    let token = create_session_token(secret, "admin").unwrap();
    assert!(!verify_session_token(secret, "attacker", &token));
}

#[test]
fn test_verify_rejects_tampered_signature() {
    let secret = "test_secret";
    let token = create_session_token(secret, "admin").unwrap();
    let parts: Vec<&str> = token.splitn(3, '.').collect();
    let tampered = format!("{}.{}.{}", parts[0], parts[1], "a".repeat(64));
    assert!(!verify_session_token(secret, "admin", &tampered));
}

#[test]
fn test_verify_rejects_expired_token() {
    let secret = "test_secret";
    let old_ts: u64 = 1000;
    let nonce = "abcdef1234567890abcdef1234567890"; // 32 hex chars
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(format!("astral:admin:session:{}:{}", old_ts, nonce).as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    let expired_token = format!("{}.{}.{}", old_ts, nonce, sig);
    assert!(!verify_session_token(secret, "admin", &expired_token));
}

#[test]
fn test_verify_rejects_malformed_tokens() {
    let secret = "s";
    assert!(!verify_session_token(secret, "u", ""));
    assert!(!verify_session_token(secret, "u", "no_dot"));
    assert!(!verify_session_token(secret, "u", "."));
    assert!(!verify_session_token(secret, "u", ".."));
    assert!(!verify_session_token(secret, "u", "notanumber.nonce.abc"));
    assert!(!verify_session_token(secret, "u", "123.nonce.not_hex_$$"));
    // Old 2-part format (no nonce) should be rejected
    assert!(!verify_session_token(secret, "u", "123.abc"));
}

#[test]
fn test_different_users_get_different_tokens() {
    let secret = "shared_secret";
    let t1 = create_session_token(secret, "alice").unwrap();
    let t2 = create_session_token(secret, "bob").unwrap();
    let sig1 = t1.splitn(3, '.').nth(2).unwrap();
    let sig2 = t2.splitn(3, '.').nth(2).unwrap();
    assert_ne!(sig1, sig2);
}

#[test]
fn test_concurrent_logins_produce_different_tokens() {
    // H-1: Two logins for the same user at the same second must differ due to nonce.
    let secret = "test_secret";
    let t1 = create_session_token(secret, "admin").unwrap();
    let t2 = create_session_token(secret, "admin").unwrap();
    let nonce1 = t1.splitn(3, '.').nth(1).unwrap();
    let nonce2 = t2.splitn(3, '.').nth(1).unwrap();
    assert_ne!(nonce1, nonce2, "Nonces must be unique across concurrent logins");
}

// --- Rate Limiter Tests ---

#[tokio::test]
async fn test_limiter_allows_under_limit() {
    let limiter = LoginLimiter::new();
    let ip: std::net::IpAddr = "10.0.0.1".parse().unwrap();
    for _ in 0..MAX_LOGIN_ATTEMPTS {
        assert!(limiter.check_rate_limit(ip).await);
    }
}

#[tokio::test]
async fn test_limiter_blocks_over_limit() {
    let limiter = LoginLimiter::new();
    let ip: std::net::IpAddr = "10.0.0.1".parse().unwrap();
    for _ in 0..MAX_LOGIN_ATTEMPTS {
        limiter.check_rate_limit(ip).await;
    }
    assert!(!limiter.check_rate_limit(ip).await, "Should block after max attempts");
}

#[tokio::test]
async fn test_limiter_isolates_by_ip() {
    let limiter = LoginLimiter::new();
    let ip1: std::net::IpAddr = "10.0.0.1".parse().unwrap();
    let ip2: std::net::IpAddr = "10.0.0.2".parse().unwrap();
    for _ in 0..MAX_LOGIN_ATTEMPTS {
        limiter.check_rate_limit(ip1).await;
    }
    assert!(!limiter.check_rate_limit(ip1).await);
    assert!(limiter.check_rate_limit(ip2).await);
}

// --- Password Hashing Tests ---

#[test]
fn test_argon2_hash_and_verify() {
    let password = "correct_password";
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    let parsed = PasswordHash::new(&hash).unwrap();
    assert!(Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok());
}

#[test]
fn test_argon2_rejects_wrong_password() {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(b"correct", &salt)
        .unwrap()
        .to_string();
    let parsed = PasswordHash::new(&hash).unwrap();
    assert!(Argon2::default().verify_password(b"wrong", &parsed).is_err());
}

#[test]
fn test_argon2_different_salts_produce_different_hashes() {
    let password = b"same_password";
    let h1 = Argon2::default()
        .hash_password(password, &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();
    let h2 = Argon2::default()
        .hash_password(password, &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();
    assert_ne!(h1, h2);
}

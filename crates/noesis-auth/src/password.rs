use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use noesis_core::EngineError;

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String, EngineError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| EngineError::AuthError(format!("Password hashing failed: {}", e)))?;
    
    Ok(password_hash.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, EngineError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|e| EngineError::AuthError(format!("Invalid password hash: {}", e)))?;
    
    let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);
    
    Ok(result.is_ok())
}

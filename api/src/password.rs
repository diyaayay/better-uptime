use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, Error};
use rand_core::OsRng; 

/// Hash a password 
/// Generates a random salt, hashes the password with salt, and returns the hashed pwd
pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

    Ok(password_hash)
}


/// Verify a plaintext password against stored hash

pub fn verify_password(password: &str, hash:&str) -> Result<bool, Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash){
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
    
}
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;

fn main() {
    let password = "123456";
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    let hashed = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    println!("{}", hashed.to_string());
}




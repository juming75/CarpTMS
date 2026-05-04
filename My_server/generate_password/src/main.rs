use bcrypt::{hash, DEFAULT_COST};

fn main() {
    let password = "admin123";
    let hashed_password = hash(password, DEFAULT_COST).unwrap();
    println!("Password: {}", password);
    println!("Hashed password: {}", hashed_password);
}




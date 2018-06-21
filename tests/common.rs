use std::env;

#[test]
pub fn integration() {

    println!("Welcome to EDB");
    println!("Running Integration Tests in {:?}", env::current_dir().unwrap());
    /* Start TestRPC, stuff like that here */
}

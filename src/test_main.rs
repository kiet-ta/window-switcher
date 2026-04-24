#[test]
fn test_tokio() {
    println!("Testing tokio");
    if let Ok(runtime) = tokio::runtime::Runtime::new() {
        println!("Tokio works");
    } else {
        println!("Tokio failed");
    }
}

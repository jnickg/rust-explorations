fn print_message(msg: &str) -> usize {
    println!("{}", msg);
    msg.len()
}

fn main() {
    let len = print_message("Hello, world!");
    println!("Message length: {}", len);
}

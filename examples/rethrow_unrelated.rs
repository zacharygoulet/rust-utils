use rust_utils::raise_handler;

#[raise_handler]
fn main() {
    panic!("this is not a Raised value");
}

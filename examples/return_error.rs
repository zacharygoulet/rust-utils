use rust_utils::raise_handler;

#[raise_handler]
fn main() -> Result<(), std::io::Error> {
    Err(std::io::Error::other("ordinary main error"))
}

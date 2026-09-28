use rust_utils::raise::RaiseExt;
use rust_utils::raise_handler;

#[raise_handler]
fn main() {
    Err::<(), _>("the operation failed").raise();
}

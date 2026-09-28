# Raise utilities

The `raise` module provides an opt-in, panic-based way to propagate selected
errors through a call stack. It is intended for cases where you want exception-like
handling, without changing how unrelated Rust panics behave.

## Raise an error from `Result`

Import `RaiseExt` to make `.raise()` available on any `Result<T, E>` where `E`
implements `Display`:

```rust
use rust_utils::raise::RaiseExt;

fn main() {
    let value = Ok::<_, &str>(42).raise();
    assert_eq!(value, 42);

    Err::<(), _>("something went wrong").raise();
}
```

On `Ok`, `.raise()` returns the value. On `Err`, it formats the error into a
`String` and panics with a `Raised` payload. The original error value and type are
not retained.

## Handle raised errors from `main`

Use `#[raise_handler]` on a synchronous, parameterless `fn main()`. It catches
`Raised` panics, prints their message, and exits with status 1:

```rust
use rust_utils::raise::RaiseExt;
use rust_utils::raise_handler;

#[raise_handler]
fn main() {
    Err::<(), _>("something went wrong").raise();
}
```

The attribute also supports ordinary main return types, such as
`Result<(), std::io::Error>`. If no `Raised` panic occurs, the original return
value is passed back to Rust unchanged, so normal `main` error reporting still
works. Other panics are resumed and retain their usual behavior.

The attribute installs a process-global panic hook that suppresses output only
for `Raised` payloads and delegates all other panics to the previous hook. The
hook is global because Rust panic hooks are process-global.

## Catch manually

For finer-grained control, use `catch_raised!`. It returns `Ok(value)` if the
closure completes, or `Err(Raised)` if it panics with a `Raised` payload. Any
other panic is resumed:

```rust
use rust_utils::catch_raised;
use rust_utils::raise::RaiseExt;

fn main() {
    let result = catch_raised!({
        Err::<(), _>("something went wrong").raise();
    });

    match result {
        Ok(()) => println!("Nothing happened"),
        Err(error) => println!("{error}"),
    }
}
```

`catch_raised!` inherits `catch_unwind`'s `UnwindSafe` requirement and only works
when the program is built with panic unwinding, not `panic = "abort"`. It does
not install a panic hook. To suppress `Raised` output when catching manually, call
`rust_utils::raise::suppress_raised_hook()` before raising. This changes the
process-global hook and delegates unrelated panics to the previously installed
hook.

## Runnable examples

From the repository root, run:

```sh
cargo run --example handle_raised
cargo run --example rethrow_unrelated
cargo run --example return_error
```

`handle_raised` handles a raised value and exits with status 1.
`rethrow_unrelated` triggers an ordinary panic, which is not intercepted and
retains the standard panic output and exit behavior. `return_error` returns an
ordinary `io::Error` from `main`, which Rust reports through its normal
`Termination` behavior.

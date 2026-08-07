# Tests

The recommended tool: cargo-llvm-cov

cargo install cargo-llvm-cov
cargo llvm-cov              # text summary, per-file %
cargo llvm-cov --html      # generates an HTML report...
cargo llvm-cov --open      # ...and opens it in your browser

cargo test -- --list
---
description: Run formatting and lint checks before pushing to GitHub
---
1. Run `cargo +nightly fmt --check`.
// turbo
2. Run `cargo clippy --all-targets --all-features -- -D warnings`.
3. If everything is correct, you can proceed with `git push`.

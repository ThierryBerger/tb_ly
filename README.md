# My lightyear sandbox

This takes examples from [lightyear](https://github.com/cBournhonesque/lightyear) and mashes them up together.

- Visibility
- Authentication
  - Fixed with clients unaware of secret.
- Map loading (bonus)
- leafwing input

## Opinionated decisions

- Removed client hosted for simplicity
- Separated client and server for clarity (no feature gating)
- Rust based certificate generation (see https://github.com/cBournhonesque/lightyear/pull/1378)

## Hot to run

### Setup

Generate certificates from:

- `cd server && cargo run --bin generate_cert_self_signed`
  - put resulting files in `./certificates/`
- `cd server ./generate_auth_private_key.sh`

### Run

- `cd crates/server && cargo run --bin server`
- `cd crates/client && cargo run`

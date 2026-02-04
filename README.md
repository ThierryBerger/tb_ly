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
- Use fly.io for server deployment
- Use itch.io for client deployment (+ bevy ci template)

## How to run

### Setup

Generate certificates from:

- `cargo run -p server --bin generate_cert_self_signed`
  - put resulting files in `./certificates/`
  - digest should be copied in an env var in `crates/client/.env` (see `.env.example`)
- `./crates/server/generate_auth_private_key.sh`
  - it's optional, but important to crypt authentication token.

### Run

- `cargo run --bin server --features local`
  - From root because hardcoded path to tls certificates.
    - `certificates/` should probably be in `crates/server/`, and we run the binary from there, but `certificates/` shouldn't be in a parent folder (`..`), to help with fly.io publish.
  - Have the (optional) private key in same folder as you run the command
- `cd crates/client && cargo run --features local`
  - from `crates/client` because of `.env`, which we don't want to mix with server.
- Running without `local` feature should work, `CERT_DIGEST` may have to be populated,
  it's not a secret so we **could** publish it, but it's tied to tls certificates which should be secrets,
  so for consistency I handled digest as a secret too.

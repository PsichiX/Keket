list:
  just --list

format:
  cargo fmt --all

build:
  cargo build --all --all-features

test:
  just redb-pack
  just fjall-pack
  cargo test --all --all-features

miri:
  cargo +nightly miri test --manifest-path ./crates/_/Cargo.toml

clippy:
  cargo clippy --all --all-features
  cargo clippy --tests --all --all-features

checks:
  just format
  just build
  just clippy
  just test
  just miri

redb-pack:
  cargo run --manifest-path ./crates/redb-pack/Cargo.toml

fjall-pack:
  cargo run --manifest-path ./crates/fjall-pack/Cargo.toml

pack:
  just redb-pack
  just fjall-pack

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

list-outdated:
  cargo outdated -R -w

update:
  cargo update --manifest-path ./crates/_/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/http/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/redb/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/redb-pack/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/fjall/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/fjall-pack/Cargo.toml --aggressive

publish:
  cargo publish --no-verify --manifest-path ./crates/_/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/http/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/redb/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/fjall/Cargo.toml
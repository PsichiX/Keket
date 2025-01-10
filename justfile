list:
  just --list

format:
  cargo fmt --all

build:
  cargo build --all --all-features

test:
  cargo test --all --all-features

miri:
  cargo +nightly miri test --manifest-path ./crate/Cargo.toml

clippy:
  cargo clippy --all --all-features
  cargo clippy --tests --all --all-features

checks:
  just format
  just build
  just clippy
  just test
  just miri

demo:
  cargo run --release --manifest-path ./demo/Cargo.toml

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

list-outdated:
  cargo outdated -R -w

update:
  cargo update --manifest-path ./crate/Cargo.toml --aggressive

publish:
  cargo publish --no-verify --manifest-path ./crate/Cargo.toml
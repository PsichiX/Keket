list:
  just --list

format:
  cargo fmt --all

build:
  cargo build --all --all-features

test:
  just redb-pack
  cargo test --all --all-features

clippy:
  cargo clippy --all --all-features
  cargo clippy --tests --all --all-features

checks:
  just format
  just build
  just clippy
  just test

redb-pack:
  cargo run --manifest-path ./crates/redb-pack/Cargo.toml

hot-reloading:
  cargo run --manifest-path ./crates/_/Cargo.toml --example 07_hot_reloading

ingame:
  cargo run --manifest-path ./crates/_/Cargo.toml --example 08_ingame

server:
  cd ./resources/ && cargo run --manifest-path ../crates/server/Cargo.toml

clean:
  find . -name target -type d -exec rm -r {} +
  just remove-lockfiles

remove-lockfiles:
  find . -name Cargo.lock -type f -exec rm {} +

list-outdated:
  cargo outdated -R -w

update:
  cargo update --manifest-path ./crates/_/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/graph-derive/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/graph/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/http/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/client/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/redb/Cargo.toml --aggressive
  cargo update --manifest-path ./crates/redb-pack/Cargo.toml --aggressive

book:
  mdbook build book

book-dev:
  mdbook watch book --open

publish:
  cargo publish --no-verify --manifest-path ./crates/_/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/graph-derive/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/graph/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/http/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/client/Cargo.toml
  cargo publish --no-verify --manifest-path ./crates/redb/Cargo.toml

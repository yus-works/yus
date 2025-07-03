# 0 – common base with toolchain
FROM rust:1.85-slim AS base
RUN apt-get update \
 && apt-get install -y pkg-config libssl-dev ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# global Cargo/registry cache for all later stages
ENV CARGO_HOME=/usr/local/cargo
ENV CARGO_TARGET_DIR=/app/target

RUN --mount=type=cache,id=registry,target=$CARGO_HOME/registry \
    --mount=type=cache,id=git-db,target=$CARGO_HOME/git \
    cargo install cargo-chef --locked \
 && cargo install trunk --locked \
 && rustup target add wasm32-unknown-unknown

# 1 – dependency graph
FROM base AS planner
WORKDIR /app

# copy only manifests
COPY Cargo.toml Cargo.lock Trunk.toml ./
COPY ui/Cargo.toml       ./ui/
COPY site/Cargo.toml     ./site/

# dummy targets so `cargo metadata` is happy.
RUN mkdir -p ui/src site/src \
 && printf 'fn main() {}\n' > site/src/main.rs \
 && printf '// stub lib\n'   > ui/src/lib.rs

RUN cargo chef prepare --recipe-path recipe.json

# 2 – build all dependencies
FROM base AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json .

RUN --mount=type=cache,id=registry,target=$CARGO_HOME/registry \
    --mount=type=cache,id=git-db,target=$CARGO_HOME/git \
    cargo chef cook --release --recipe-path recipe.json \
    --target wasm32-unknown-unknown \
 && cargo chef cook --release --recipe-path recipe.json


# 2a – cache WASM deps only
FROM base AS cacher-ui
WORKDIR /app
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --recipe-path recipe.json \
    --target wasm32-unknown-unknown \
    --manifest-path ui/Cargo.toml

# 2b – cache native deps
FROM base AS cacher-site
WORKDIR /app
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --recipe-path recipe.json \
    --package site

# 3 – build the actual project
FROM base AS builder
WORKDIR /app

# re-use both dependency layers
COPY --from=cacher-ui   /app/target /app/target
COPY --from=cacher-site /app/target /app/target
COPY . .
RUN trunk build --release --public-url /pkg --dist ./dist
WORKDIR /app/site
RUN cargo build --release

# 4 – final runtime image
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/site ./site
COPY --from=builder /app/dist                ./dist
COPY --from=builder /app/ui/assets           ./assets
ENV PORT=3000
EXPOSE 3000
CMD ["./site"]

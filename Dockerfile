####################  Stage 1 – build UI  ####################
FROM rust:1.85 AS build-ui
RUN rustup target add wasm32-unknown-unknown && \
    cargo install trunk --locked
WORKDIR /app
COPY ui ./ui
COPY Cargo.toml Cargo.lock ./

COPY site/Cargo.toml ./site/
RUN trunk build --release \
        --public-url /pkg \
        --dist ./target/site

####################  Stage 2 – build server  #################
FROM rust:1.85 AS build-site
WORKDIR /app
COPY . .
RUN cargo build -p site --release

####################  Stage 3 – final image  ##################
FROM debian:bookworm-slim
WORKDIR /app
# copy server binary
COPY --from=build-site /app/target/release/site ./site
# copy static assets folder produced by trunk
COPY --from=build-ui   /app/target/site ./target/site
ENV LEPTOS_SITE_ROOT=/app/target/site
EXPOSE 3000
CMD ["./site"]

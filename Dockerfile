####################  Stage 1 – build UI  ####################
FROM rust:1.85 AS build-ui
RUN rustup target add wasm32-unknown-unknown \
 && cargo install trunk --locked

WORKDIR /app
COPY . .
RUN trunk build --release \
        --public-url /pkg \
        --dist ./dist

####################  Stage 2 – build server  #################
FROM rust:1.85 AS build-site
WORKDIR /app
COPY . .
RUN cargo build -p site --release

####################  Stage 3 – final image  ##################
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build-site /app/target/release/site ./site
COPY --from=build-ui   /app/dist              ./dist
COPY --from=build-ui   /app/ui/assets         ./assets
ENV PORT=3000
EXPOSE 3000
CMD ["./site"]


# Stage 1: Build app
FROM rust:1.85 AS builder

# Install the WASM target and Trunk
RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk --locked

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY Trunk.toml index.html ./
COPY src ./src

RUN trunk build --release

# Stage 2: Serve with nginx
FROM nginx:alpine

# Clean default nginx content
RUN rm -rf /usr/share/nginx/html/*

# Copy the static site from the build stage
COPY --from=builder /app/dist/ /usr/share/nginx/html/

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]

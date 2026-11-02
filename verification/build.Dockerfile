FROM rust:1.93.1-slim-bookworm
RUN rustup target add wasm32-wasip2 && rustup component add rustfmt
WORKDIR /source

FROM rust:1.94-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc

COPY --from=builder /app/target/release/blitz /usr/local/bin/blitz

ENV BIND_ADDR=0.0.0.0:3000

EXPOSE 3000

CMD ["/usr/local/bin/blitz"]

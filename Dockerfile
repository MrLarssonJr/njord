FROM lukemathwalker/cargo-chef:latest-rust-1 as chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
COPY --from=builder /app/target/release/njord /usr/local/bin/njord
ENTRYPOINT ["njord"]

FROM rust:latest as builder
WORKDIR /usr/src/njord
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/njord /usr/local/bin/njord
CMD ["njord"]

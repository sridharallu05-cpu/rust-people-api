FROM rust:1.93 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /app/target/release/rust_playground .

EXPOSE 3000

CMD ["./rust_playground"]

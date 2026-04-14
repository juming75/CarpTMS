FROM rust:1.70 as builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

WORKDIR /app

COPY --from=builder /app/target/release/carptms_server .

EXPOSE 8082

CMD ["./carptms_server"]

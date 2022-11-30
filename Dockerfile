FROM rust:1.64-buster as builder

WORKDIR /usr/src/rs_chat
COPY . .

RUN cargo build --release --bin server

FROM debian:buster-slim
COPY --from=builder /usr/src/rs_chat/target/release/server /usr/local/bin/server

CMD ["server", "-a", "[::]:8080"]

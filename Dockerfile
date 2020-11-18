FROM rust:1.47 as builder

# Prepare dep cache layer
RUN USER=root cargo new --bin app
WORKDIR /app

COPY ./Cargo.toml ./Cargo.lock ./
RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

# Build for release
RUN rm ./target/release/deps/kylin_data_proxy*
RUN cargo build --release


FROM debian:buster-slim

COPY --from=builder /app/target/release/kylin_data_proxy ${APP}/kylin_data_proxy

EXPOSE 8080

CMD ["./kylin_data_proxy"]

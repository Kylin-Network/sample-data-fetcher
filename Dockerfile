FROM paritytech/ci-linux:production as builder

# Prepare dep cache layer
RUN USER=root cargo new --bin app
WORKDIR /app

COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
#COPY ./ustc.config /root/.cargo/config
#RUN cat ~/.cargo/config
RUN cargo build


FROM ubuntu
RUN apt-get update && apt-get install ca-certificates -y
COPY --from=builder /app/target/debug/data_fetcher /data_fetcher

EXPOSE 8080

#CMD ["./data_fetcher"]
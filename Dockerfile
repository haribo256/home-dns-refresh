FROM rust:1-slim-bullseye as builder
RUN mkdir /app
WORKDIR /app
COPY . /app
RUN cargo build --release
RUN ls -R

FROM debian:bullseye-slim
RUN apt-get update -y
RUN apt-get install ca-certificates -y
RUN mkdir /app
ENV PATH="${PATH}:/app"
WORKDIR /app
COPY --from=builder /app/target/release/home-dns-refresh /app
LABEL org.opencontainers.image.description "Updates an Azure DNS zone A record to the currently external IP address"
ENTRYPOINT ["home-dns-refresh"]
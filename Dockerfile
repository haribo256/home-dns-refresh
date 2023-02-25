FROM rust:1-slim-bullseye as toolchain
RUN mkdir -p /dummy/src
WORKDIR /dummy
RUN apt-get update -y
RUN apt-get install build-essential pkg-config libssl-dev -y
RUN /bin/bash -c "echo -e '[package]\nname=\"dummy\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[dependencies]\nsmallest-uint=\"0.1\"\n' > Cargo.toml"
RUN /bin/bash -c "echo -e 'fn main() { println!(\"Dummy\"); }' > src/main.rs"
RUN cat Cargo.toml
RUN cat src/main.rs
RUN cargo fetch

FROM toolchain as builder
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
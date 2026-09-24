FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --bin onyx-node && strip target/release/onyx-node

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 onyx
WORKDIR /home/onyx
COPY --from=build /src/target/release/onyx-node /usr/local/bin/onyx-node
USER onyx
VOLUME ["/home/onyx/state", "/home/onyx/status"]
# libp2p TCP + QUIC for `listen`
EXPOSE 4710/tcp 4710/udp
ENTRYPOINT ["onyx-node"]
CMD ["listen", "--topic", "nexus/mesh/v0", "--interval", "30"]

FROM rust:1.81-bookworm AS build
WORKDIR /src
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release --bin onyx-node

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 onyx
WORKDIR /home/onyx
COPY --from=build /src/target/release/onyx-node /usr/local/bin/onyx-node
USER onyx
VOLUME ["/home/onyx/state", "/home/onyx/status"]
ENTRYPOINT ["onyx-node"]
CMD ["pulse"]

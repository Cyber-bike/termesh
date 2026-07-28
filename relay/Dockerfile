# syntax=docker/dockerfile:1
#
# Single-node relay image (doc 6.1).
#
# NOT BUILT ON THE DEVELOPMENT HOST: there is no Docker daemon in the loop, so
# this file has been reviewed but never executed. Build it once before relying
# on it.

# --- build ------------------------------------------------------------------
FROM rust:1.97.1-bookworm AS build

WORKDIR /src

# The protocol crate is a path dependency, so it has to be in the context.
COPY rust-toolchain.toml ./
COPY protocol/generated/rust ./protocol/generated/rust
COPY protocol/generated/protocol.bundle.schema.json ./protocol/generated/
COPY protocol/fixtures ./protocol/fixtures
COPY relay ./relay

WORKDIR /src/relay
RUN cargo build --release --locked

# --- runtime ----------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# ca-certificates is required even though the relay does not make outbound TLS
# calls today: sqlx and any future egress would fail confusingly without it.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Doc 13.5: nothing runs as root.
RUN useradd --system --create-home --uid 10001 --shell /usr/sbin/nologin termy

COPY --from=build /src/relay/target/release/termy-relay /usr/local/bin/termy-relay

# Doc 6.1 and review item B9: the SQLite file must live on a volume, otherwise a
# container restart silently destroys every account, pairing code and device.
RUN mkdir -p /var/lib/termy-relay && chown termy:termy /var/lib/termy-relay
VOLUME ["/var/lib/termy-relay"]

USER termy
WORKDIR /var/lib/termy-relay

ENV TERMY_DB_PATH=/var/lib/termy-relay/relay.db \
    TERMY_BIND=0.0.0.0:8080 \
    RUST_LOG=termy_relay=info,tower_http=info

# TERMY_PEPPER, TERMY_JWT_SECRET and TERMY_RELAY_URL have no safe defaults and
# are required at startup; the process refuses to boot without them.

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/termy-relay", "healthcheck", "--addr", "127.0.0.1:8080"]

ENTRYPOINT ["/usr/local/bin/termy-relay"]
CMD ["serve"]

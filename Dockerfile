FROM rust:1.41-slim

EXPOSE 8080

RUN apt-get update -y && \
  apt-get install -y --no-install-recommends \
  build-essential cmake pkg-config openssl libssl-dev && \
  rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build $cargo_build_flags && \
  cp target/*/libindy_vdr.so . && \
  cp target/*/indy-vdr-proxy vdr-proxy

CMD ./vdr-proxy -g https://raw.githubusercontent.com/sovrin-foundation/sovrin/stable/sovrin/pool_transactions_sandbox_genesis -p 8080
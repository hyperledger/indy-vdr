
FROM rust:1.41-slim

ADD ./ca-certificates.crt /usr/share/ca-certificates/netskope/
RUN dpkg-reconfigure ca-certificates
RUN update-ca-certificates
ENV SSL_CERT_DIR /usr/share/ca-certificates/netskope
ENV SSL_CERT_FILE /usr/share/ca-certificates/netskope/ca-certificates.crt
ENV GIT_SSL_NO_VERIFY true
EXPOSE 8080

RUN apt-get update -y && \
  apt-get install -y --no-install-recommends \
  build-essential cmake pkg-config openssl libssl-dev && \
  rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build $cargo_build_flags && \
  cp target/*/libindy_vdr.so ./bin/libindy_vdr.so && \
  cp target/*/indy-vdr-proxy ./bin/vdr-proxy

ENV PATH "$PATH:$PWD/bin"

CMD vdr-proxy -g https://raw.githubusercontent.com/sovrin-foundation/sovrin/stable/sovrin/pool_transactions_sandbox_genesis -p 8080
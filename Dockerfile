FROM ubuntu:23.10

WORKDIR /app

RUN apt-get update && apt-get install -y curl && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update && \
    apt-get install -y wget pkg-config libssl-dev build-essential

RUN wget https://fastdl.mongodb.org/tools/db/mongodb-database-tools-ubuntu2204-x86_64-100.9.0.deb && \
    apt install -y ./mongodb-database-tools-*.deb && \
    rm -f mongodb-database-tools-*.deb

RUN apt-get install -y protobuf-compiler

COPY Cargo.toml config.toml ./
COPY src ./src

RUN cargo build --release

EXPOSE 8000
ENV RUST_BACKTRACE "1"
CMD ["./target/release/indexer"]

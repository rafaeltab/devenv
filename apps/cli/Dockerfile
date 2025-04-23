FROM rust:1.77

WORKDIR /usr/src/myapp
COPY src/ src/
COPY Cargo.lock .
COPY Cargo.toml .

RUN cargo build

COPY tests/ tests/

ENTRYPOINT [ "cargo",  "test", "--color", "always", "--test", "integration_test" ]


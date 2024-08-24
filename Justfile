build:
    cargo build

release:
    cargo build --release

test:
    cargo test -- --skip integration_test

integration_test:
    docker build -t rafaeltab_cli_integration_tests .
    docker run rafaeltab_cli_integration_tests

activate: release
    sudo cp target/release/rafaeltab /usr/local/bin/

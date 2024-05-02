FROM rust:latest
WORKDIR /myapp

COPY . /tiler
WORKDIR /tiler
RUN rustup override set nightly
RUN cargo build --release
CMD ["/tiler/target/release/jnickg_rust_explorations"]

EXPOSE 3000
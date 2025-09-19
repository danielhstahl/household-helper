FROM rust:1.90.0 AS builder
WORKDIR /app
# copy contents of draid into code
ADD draid /app/
ENV SQLX_OFFLINE=true
RUN curl -L https://github.com/cross-rs/cross/releases/download/v0.2.5/cross-x86_64-unknown-linux-gnu.tar.gz -o cross-x86_64-unknown-linux-gnu.tar.gz
RUN tar -xvzf cross-x86_64-unknown-linux-gnu.tar.gz
RUN rustup update stable
RUN rustup target add x86_64-unknown-linux-musl
RUN ./cross build --release --target x86_64-unknown-linux-musl
FROM scratch
COPY --from=builder --chown=app:app /app/target/x86_64-unknown-linux-musl/release/draid /draid
CMD ["/draid"]

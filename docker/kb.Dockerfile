FROM scratch
ADD knowledge-base/target/x86_64-unknown-linux-musl/release/knowledge-base /knowledge-base
CMD ["/knowledge-base"]

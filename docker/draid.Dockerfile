FROM scratch
RUN mkdir -p /tmp # store uploaded files for KBs
ADD draid/target/x86_64-unknown-linux-musl/release/draid /draid
CMD ["/draid"]

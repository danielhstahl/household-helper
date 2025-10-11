FROM busybox AS build-env
RUN mkdir -p /tmp

FROM scratch
# store uploaded files for KBs
COPY --from=build-env /tmp /tmp
ADD draid/target/x86_64-unknown-linux-musl/release/draid /draid
CMD ["/draid"]

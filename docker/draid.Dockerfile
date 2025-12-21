#FROM busybox AS build-env
#RUN mkdir -p /tmp

FROM rust:1.92.0-alpine3.22 as build
WORKDIR /app
#RUN mkdir -p /tmp
# These ARGs are provided automatically by Buildx
ARG TARGETARCH

# Install the cross-compilation target and linker for ARM64
RUN if [ "$TARGETARCH" = "arm64" ]; then \
    rustup target add aarch64-unknown-linux-musl; \
    fi

# Build the actual application
COPY draid/ .
RUN if [ "$TARGETARCH" = "arm64" ]; then \
    cargo build --release --target aarch64-unknown-linux-musl; \
    cp target/aarch64-unknown-linux-musl/release/draid /draid; \
    else \
    cargo build --release; \
    cp target/release/draid /draid; \
    fi

FROM scratch
# store uploaded files for KBs
#COPY --from=build /tmp /tmp
COPY --from=build /draid /draid
CMD ["/draid"]

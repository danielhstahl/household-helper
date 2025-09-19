FROM scratch
ADD draid/target/x86_64-unknown-linux-musl/release/draid /draid
#COPY --from=builder --chown=app:app /app/target/x86_64-unknown-linux-musl/release/draid /draid
CMD ["/draid"]

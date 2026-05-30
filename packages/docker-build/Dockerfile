FROM megaton-docker/builder
LABEL org.opencontainers.image.source=https://github.com/Pistonite/megaton

ARG GCC_VERSION
ARG RUST_VERSION

COPY print-info.sh /root/print-info.sh

# Install toolchain with megaton-toolchain-build
# We are not using the main megaton command directly to install the toolchain
# because this way, the toolchain only needs to be rebuilt if toolchain-build changes,
# not the main build tool package or the library
ENV MEGATON_HOME=/opt/megaton
COPY temp/toolchain-build /opt/megaton/toolchain-build
# Caching the rust directory for LLVM build
# Caching cargo directory for cargo build
RUN \
--mount=type=cache,target=/opt/megaton/rust,id=megaton-docker-build-llvm-${GCC_VERSION} \
--mount=type=cache,target=/root/.cargo/registry,id=megaton-docker-build-cargo-${RUST_VERSION} \
cargo run \
--manifest-path /opt/megaton/toolchain-build/Cargo.toml \
--example docker_install \
--release \
-- -v

# Install megaton-cmd
COPY temp/megaton-cmd /opt/megaton/megaton-cmd
RUN cargo install \
  --path /opt/megaton/megaton-cmd \
  --locked && \
  rm -rf /opt/megaton/megaton-cmd/target


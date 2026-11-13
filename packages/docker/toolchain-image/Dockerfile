FROM megaton/builder-base

ARG GCC_VERSION
ARG RUST_VERSION

# Install toolchain with megaton-toolchain
# We are not using the main megaton command directly to install the toolchain
# because this way, the toolchain only needs to be rebuilt if toolchain or config changes
# not the main build tool package or the library
COPY ./crates /opt/megaton/toolchain-build
ENV MEGATON_HOME=/opt/megaton
RUN \
--mount=type=cache,target=/opt/megaton/rust,id=megaton-docker-build-llvm-${GCC_VERSION} \
--mount=type=cache,target=/root/.cargo/registry,id=megaton-docker-build-cargo-${RUST_VERSION} \
cd /opt/megaton/toolchain-build && cargo run \
--bin megaton-toolchain \
-- install -v && cargo clean

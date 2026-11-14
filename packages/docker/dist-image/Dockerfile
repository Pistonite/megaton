FROM megaton/builder-toolchain
LABEL org.opencontainers.image.source=https://github.com/Pistonite/megaton
LABEL org.opencontainers.image.description="Image with DevKitA64 and Megaton installed"

ARG RUST_VERSION

COPY ./crates /opt/megaton/cmd-build
COPY ./print-info.sh /root/print-info.sh
ENV MEGATON_HOME=/opt/megaton

RUN \
--mount=type=cache,target=/opt/megaton/cmd-build/target,id=megaton-docker-cmd-build \
--mount=type=cache,target=/usr/local/cargo/registry,id=megaton-docker-build-cargo-${RUST_VERSION} \
cd /opt/megaton/cmd-build/packages/cli && cargo install --path .

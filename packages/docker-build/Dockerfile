FROM devkitpro/devkita64

# Install Rust (host)
RUN mkdir /opt/rustup
COPY temp/rustup-init.sh /opt/rustup/rustup-init.sh
RUN chmod +x /opt/rustup/rustup-init.sh && \
    /opt/rustup/rustup-init.sh -y --default-toolchain 1.95.0 --no-modify-path
ENV PATH=/root/.cargo/bin:$PATH
RUN rustc -vV

# Install toolchain with megaton-toolchain-build
# We are not using the main megaton command directly to install the toolchain
# because this way, the toolchain only needs to be rebuilt if toolchain-build changes,
# not the main build tool package or the library
ENV MEGATON_HOME=/opt/megaton
COPY temp/toolchain-build /opt/megaton/toolchain-build
RUN cargo run \
    --manifest-path /opt/megaton/toolchain-build/Cargo.toml \
    --example docker_install \
    --release \
    -- -v && \
    rustc +megaton -vV

# Install megaton-cmd
COPY temp/megaton-cmd /opt/megaton/megaton-cmd
RUN cargo install \
    --path /opt/megaton/megaton-cmd \
    --locked && \
    rm -rf /opt/megaton/megaton-cmd/target && \
    echo '==> $ megaton --version' && \
    megaton --version && \
    echo '==> $ megaton toolchain check' && \
    megaton toolchain check && \
    echo '==> $ rustup toolchain list' && \
    rustup toolchain list && \
    echo '==> $ rustup +megaton target list --installed' && \
    rustup +megaton target list --installed

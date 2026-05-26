FROM devkitpro/devkita64

ARG RUST_VERSION

# Install Rust on the host
RUN mkdir /opt/rustup
COPY ./temp/rustup-init.sh /opt/rustup/rustup-init.sh
RUN \
chmod +x /opt/rustup/rustup-init.sh && \
/opt/rustup/rustup-init.sh -y --default-toolchain ${RUST_VERSION} --no-modify-path
ENV PATH=/root/.cargo/bin:$PATH

COPY ./builder-info.sh /root/builder-info.sh

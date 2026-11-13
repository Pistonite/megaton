FROM devkitpro/devkita64

RUN apt update && apt upgrade -y

ARG RUST_VERSION

COPY ./dump-info.py /root/dump-info.py
COPY ./rustup-init.sh /root/rustup-init.sh

RUN chmod +x /root/rustup-init.sh && /root/rustup-init.sh -y --default-toolchain ${RUST_VERSION} --no-modify-path
ENV PATH=/root/.cargo/bin:$PATH


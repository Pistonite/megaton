FROM devkitpro/devkita64

ARG RUST_VERSION

RUN apt update && apt upgrade -y && apt install -y lsb-release software-properties-common

RUN wget -O /root/llvm.sh https://apt.llvm.org/llvm.sh && \
  chmod +x /root/llvm.sh && /root/llvm.sh 23
ENV PATH=/usr/lib/llvm-23/bin:$PATH
ENV LIBCLANG_PATH=/usr/lib/llvm-23/lib

COPY ./rustup-init.sh /root/rustup-init.sh
RUN chmod +x /root/rustup-init.sh && /root/rustup-init.sh -y --default-toolchain ${RUST_VERSION} --no-modify-path
ENV PATH=/root/.cargo/bin:$PATH

RUN cargo install bindgen-cli

COPY ./dump-info.py /root/dump-info.py

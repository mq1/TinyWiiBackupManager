# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

FROM debian:11

# Install dependencies
RUN apt-get update && apt-get install -y lsb-release wget software-properties-common gnupg perl make

# Install LLVM
RUN wget https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 22

# Install Rust
RUN wget https://static.rust-lang.org/rustup/rustup-init.sh && \
    chmod +x rustup-init.sh && \
    ./rustup-init.sh -y --default-toolchain 1.98 --profile minimal --component rust-src --no-modify-path

# Install i686 dependencies
RUN if [ "$(uname -m)" = "x86_64" ]; then apt-get install -y gcc-multilib; fi

ENV RUSTC_BOOTSTRAP=1
ENV CC=clang-22
ENV AR=llvm-ar-22
ENV CFLAGS="-O3 -flto"
ENV RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-22"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV PKG_CONFIG_i686_unknown_linux_gnu=i686-pc-linux-gnu-pkg-config

ENTRYPOINT [ "/root/.cargo/bin/cargo" ]

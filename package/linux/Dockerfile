# SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
# SPDX-License-Identifier: GPL-3.0-only

FROM ubuntu:20.04

# Install LLVM
RUN curl -sSL https://apt.llvm.org/llvm.sh | bash -s 22

# Install Rust
RUN curl -sSL https://static.rust-lang.org/rustup/rustup-init.sh | bash -s -- -y --default-toolchain 1.96 --profile minimal --component rust-src

# Install i686 dependencies
RUN if [ "$TARGETARCH" = "amd64" ]; then apt-get install -y gcc-multilib; fi

ENV RUSTC_BOOTSTRAP=1
ENV CC=clang-22
ENV AR=llvm-ar-22
ENV CFLAGS="-O3 -flto"
ENV RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-22"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=clang-22

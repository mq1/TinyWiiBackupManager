FROM ubuntu:20.04

# Add cross architectures
RUN dpkg --add-architecture i386 && \
    dpkg --add-architecture armhf

# Install dependencies
RUN apt-get update && apt-get install -y lsb-release wget software-properties-common gnupg libc6-dev:i386 libc6-dev:armhf

# Install LLVM 21
RUN wget https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 21 && \
    ln -sf $(which clang-21) /usr/bin/cc && \
    ln -sf $(which lld-21) /usr/bin/ld

# Install Rust
RUN wget https://static.rust-lang.org/rustup/rustup-init.sh && \
    chmod +x rustup-init.sh && \
    ./rustup-init.sh -y --default-toolchain 1.93

# Add rust to path
ENV PATH="/root/.cargo/bin:${PATH}"

# Install rust-src
RUN rustup component add rust-src


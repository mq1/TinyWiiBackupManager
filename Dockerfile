FROM rust:1.95-bullseye

ARG ARCH=x86_64

# Install LLVM
RUN wget -O /etc/apt/trusted.gpg.d/apt.llvm.org.asc https://apt.llvm.org/llvm-snapshot.gpg.key && \
    echo "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-22 main" > /etc/apt/sources.list.d/llvm.list && \
    apt-get update && \
    apt-get install -y clang-22 lld-22

# Install i686 dependencies
RUN if [ "$ARCH" = "i686" ]; then \
        dpkg --add-architecture i386 && \
        apt-get update && \
        apt-get install -y gcc-multilib pkg-config:i386 libfontconfig1-dev:i386 libssl-dev:i386; \
    fi
# Install rust-src
RUN rustup component add rust-src

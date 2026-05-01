FROM debian:bullseye

# Install dependencies
RUN apt-get update && apt-get install -y lsb-release wget software-properties-common gnupg pkg-config libfontconfig1-dev

# Install LLVM 21
RUN wget https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 22

# Install Rust
RUN wget https://static.rust-lang.org/rustup/rustup-init.sh && \
    chmod +x rustup-init.sh && \
    ./rustup-init.sh -y --default-toolchain 1.95

# Add rust to path
ENV PATH="/root/.cargo/bin:${PATH}"

# Build env vars
ENV RUSTC_BOOTSTRAP=1
ENV CC=clang-22
ENV AR=llvm-ar-22
ENV CFLAGS="-O3 -flto"
ENV RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-22"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=clang-22

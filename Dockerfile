FROM debian:bullseye

# Install dependencies
RUN apt-get update && apt-get install -y lsb-release wget software-properties-common gnupg pkg-config libfontconfig1-dev libssl-dev

# Install LLVM 22
RUN wget -qO- https://apt.llvm.org/llvm.sh | bash -s -- 22

# Install Rust
RUN wget -qO- https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.95 --component rust-src --no-modify-path

# Setup env vars
ENV PATH="/root/.cargo/bin:${PATH}"
ENV RUSTC_BOOTSTRAP=1
ENV CC=clang-22
ENV AR=llvm-ar-22
ENV CFLAGS="-O3 -flto"
ENV RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-22"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=clang-22

FROM rust:1.95-bullseye

# Install LLVM 22
RUN wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | tee /etc/apt/trusted.gpg.d/apt.llvm.org.asc && \
    add-apt-repository "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-22 main" && \
    apt-get install -y clang-22 lld-22

# Install rust-src
RUN rustup component add rust-src

# Setup env vars
ENV RUSTC_BOOTSTRAP=1
ENV CC=clang-22
ENV AR=llvm-ar-22
ENV CFLAGS="-O3 -flto"
ENV RUSTFLAGS="-C linker-plugin-lto -C link-arg=-fuse-ld=lld-22"
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang-22
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=clang-22

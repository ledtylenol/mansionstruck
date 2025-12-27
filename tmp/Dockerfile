# ---- Base image ----
FROM rust:1.92-bookworm

# ---- Install dependencies ----
RUN apt-get update && apt-get install -y --no-install-recommends \
    mingw-w64 \
    pkg-config \
    libssl-dev \
    git \
    bash \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

# ---- Add Windows target ----
RUN rustup target add x86_64-pc-windows-gnu

# ---- Configure linker for Windows ----
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

WORKDIR /app

RUN git clone https://github.com/ledtylenol/mansionstruck.git .

# ---- Build Bevy project ----
RUN cargo install cargo-binstall
RUN cargo binstall --version 0.1.0-alpha.2 --locked --no-confirm --force --git='https://github.com/TheBevyFlock/bevy_cli' bevy_cli
RUN bevy build -r --target x86_64-pc-windows-gnu
RUN cargo build --release --target x86_64-pc-windows-gnu

# ---- Copy exe to /game ----
CMD mkdir -p /game && \
    cp target/x86_64-pc-windows-gnu/release/*.exe /game
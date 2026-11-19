FROM docker.io/library/rust:latest

# Pre-requisites for PhotoCraft and standard egui/wgpu Rust apps, plus packaging tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Build tools
    cmake \
    pkg-config \
    # eframe / winit / X11 / Wayland dependencies
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libegl1-mesa-dev \
    # wgpu dependencies
    libvulkan-dev \
    # Audio (useful for FilmCraft, etc.)
    libasound2-dev \
    # Packaging tools (AppImage, deb, rpm)
    desktop-file-utils \
    appstream \
    zsync \
    libfuse2 \
    curl \
    ca-certificates \
    # Flatpak tools
    flatpak \
    flatpak-builder \
    librsvg2-common \
    && rm -rf /var/lib/apt/lists/*

# Install nfpm for DEB / RPM packaging
RUN curl -sfL https://raw.githubusercontent.com/goreleaser/nfpm/main/scripts/install.sh | sh -s -- -b /usr/local/bin

# Add the flathub repository so flatpak builds can pull runtimes
RUN flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

# Set default cargo target dir to avoid permission issues when mounting from host
ENV CARGO_TARGET_DIR=/tmp/target

WORKDIR /workspace
CMD ["/bin/bash"]

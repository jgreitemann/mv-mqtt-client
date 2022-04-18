FROM fedora:36

RUN dnf groupinstall -y "Development Tools"

RUN dnf install -y \
    rust \
    cargo \
    cmake \
    meson \
    openssl-devel \
    gtk4-devel \
    libadwaita-devel


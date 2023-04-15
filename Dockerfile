# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): slim-buster, bullseye
ARG VARIANT="bullseye"

# Use an official Python runtime as a parent image
FROM python:3.11.3-${VARIANT}

# [Option] Install zsh
ARG INSTALL_ZSH="true"
# [Option] Upgrade OS packages to their latest versions
ARG UPGRADE_PACKAGES="true"

# Install needed packages and setup non-root user. Use a separate RUN statement to add your own dependencies.
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Install required packages
RUN apt-get update && \
    apt-get install -y curl git

# Install Rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Add Rust to the PATH environment variable
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Rust nightly-2023-01-24
RUN rustup install nightly-2023-01-24

# Install Buck2 with Cargo
RUN cargo +nightly-2023-01-24 install --git https://github.com/facebook/buck2.git buck2

# Start an interactive shell (zsh if installed, bash otherwise)
CMD ["/bin/sh", "-c", "if [ -x /usr/bin/zsh ]; then exec /usr/bin/zsh; else exec /bin/bash; fi"]

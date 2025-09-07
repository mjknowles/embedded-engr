FROM mcr.microsoft.com/devcontainers/rust:2.0.2-bullseye

# Install additional system packages
# Install GitHub CLI repository
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | gpg --dearmor -o /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        git \
        curl \
        build-essential \
        pkg-config \
        libssl-dev \
        usbutils \
        udev \
        openocd \
        gh \
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Switch to vscode user for installations
USER vscode

# Install Claude Code CLI
RUN curl -fsSL https://claude.ai/install.sh | zsh

# Ensure PATH includes cargo and claude
ENV PATH="/home/vscode/.local/bin:/home/vscode/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /workspace
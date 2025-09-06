FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Add completions
RUN echo "source /usr/share/bash-completion/completions/git" >> /home/vscode/.bashrc
RUN echo "source <( rustup completions bash )" >> /home/vscode/.bashrc
RUN echo "source <( rustup completions bash cargo )" >> /home/vscode/.bashrc

RUN rustup component add rustfmt clippy

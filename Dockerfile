FROM rust:1.85.0-slim-bookworm

RUN apt-get update -y && \
    apt-get install -y \
    git 

ARG USERNAME=vscode
ARG GROUPNAME=vscode
ARG UID=1000
ARG GID=1000
RUN groupadd -g $GID $GROUPNAME && \
    useradd -m -s /bin/bash -u $UID -g $GID $USERNAME

USER ${USERNAME}

# Add completions
RUN echo "source /usr/share/bash-completion/completions/git" >> /home/vscode/.bashrc
RUN echo "source <( rustup completions bash )" >> /home/vscode/.bashrc
RUN echo "source <( rustup completions bash cargo )" >> /home/vscode/.bashrc

RUN rustup component add rustfmt clippy

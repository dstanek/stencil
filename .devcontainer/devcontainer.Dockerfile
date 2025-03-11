FROM rust:latest

RUN apt-get update && apt-get install -y \
    curl git unzip sudo \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-dist cargo-release git-cliff
RUN curl -fsSL https://raw.githubusercontent.com/nektos/act/master/install.sh | bash

WORKDIR /workspace

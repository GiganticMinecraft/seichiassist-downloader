# syntax=docker/dockerfile:1.6
FROM lukemathwalker/cargo-chef:latest-rust-1.77-slim AS chef
WORKDIR /app

FROM chef AS planner
COPY --link . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build-env
COPY --from=planner --link /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY --link . .
RUN cargo build --release

FROM ubuntu:22.04
LABEL org.opencontainers.image.source=https://github.com/GiganticMinecraft/seichiassist-downloader
RUN apt-get update -y && \
    apt-get install -y git curl gnupg openjdk-17-jdk && \
    git clone https://github.com/GiganticMinecraft/SeichiAssist.git

# sbt 公式リポジトリを追加
RUN echo "deb https://repo.scala-sbt.org/scalasbt/debian all main" | tee /etc/apt/sources.list.d/sbt.list && \
    echo "deb https://repo.scala-sbt.org/scalasbt/debian /" | tee /etc/apt/sources.list.d/sbt_old.list && \
    curl -sL "https://keyserver.ubuntu.com/pks/lookup?op=get&search=0x2EE0EA64E40A89B84B2DF73499E82A75642AC823" | apt-key add

# sbt をインストール
RUN apt-get update && \
    apt-get install -y sbt

COPY --from=build-env --link /app/target/release/seichiassist-downloader /
CMD ["./seichiassist-downloader"]


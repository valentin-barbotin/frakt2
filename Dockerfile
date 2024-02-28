FROM rust:1.74.0 as builder

RUN apt-get update \
    && apt-get install --no-install-recommends -y netcat-traditional  \
    && rm -rf /var/lib/apt/lists/*

ARG WATCH=""
RUN if [ -n "$WATCH" ]; then cargo install cargo-watch; fi;

ARG PACKAGE="server"
RUN cargo new --bin app
WORKDIR /app

ARG PROFILE="dev"

RUN cargo new --bin server
RUN cargo new --bin worker
RUN cargo new --lib shared
RUN cargo new --lib complex

COPY Cargo.* .

RUN cargo build --profile $PROFILE && rm src/*.rs

RUN ln -s debug target/dev

COPY ./tests ./tests

RUN cargo build --profile $PROFILE --package $PACKAGE

FROM debian:bookworm-slim as final

WORKDIR /app

RUN apt-get update \
    && apt-get install --no-install-recommends -y netcat-traditional  \
    && rm -rf /var/lib/apt/lists/*

ARG PROFILE="dev"
ARG PACKAGE="server"

COPY --from=builder /app/target/$PROFILE/$PACKAGE main

RUN useradd -M -s /bin/bash -u 1001 svc

RUN chown -R svc:svc /app

USER svc

CMD [ "./main" ]


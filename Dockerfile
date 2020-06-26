FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev git
WORKDIR /code
COPY . .
RUN cargo --color=always fetch
RUN cargo --color=always build --release -p playbot_irc

FROM rust:alpine as runner
RUN apk add --no-cache tini
WORKDIR /app
COPY --from=builder /code/target/release/playbot_irc .
ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/playbot_irc"]

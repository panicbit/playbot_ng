FROM alpine:edge AS base
RUN apk add --no-cache openssl
RUN apk add --no-cache libgcc

FROM base AS build
RUN apk add --no-cache cargo
RUN apk add --no-cache openssl-dev
WORKDIR /app
COPY . .
RUN cargo --color=always fetch
RUN cargo --color=always build -p playbot_irc

FROM base as run
COPY --from=build /app/target/debug/playbot_irc /bin/

ENTRYPOINT [ "/bin/playbot_irc" ]

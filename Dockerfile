FROM alpine:edge AS base
RUN apk add openssl
RUN apk add libgcc

FROM base AS build
RUN apk add cargo
RUN apk add git
RUN apk add openssl-dev
WORKDIR /app
COPY . .
RUN cargo --color=always fetch
RUN cargo --color=always build -p playbot_irc

FROM base as run
COPY --from=build /app/target/debug/playbot_irc /bin/
CMD [ "/bin/playbot_irc" ]

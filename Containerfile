ARG RUST_VERSION=1.93
ARG APP_NAME=distrust

FROM docker.io/library/rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git sqlite-dev binutils

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=schema,target=schema \
    --mount=type=bind,source=static,target=static \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server && \
    strip /bin/server

FROM docker.io/library/alpine:3.18 AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

WORKDIR /app
RUN mkdir -p /app/data && chown appuser:appuser /app/data

USER appuser

COPY --from=build /bin/server /bin/server

EXPOSE 6969

CMD ["/bin/server"]

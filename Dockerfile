FROM docker.io/library/rust:1-alpine AS builder
WORKDIR /build
RUN apk upgrade --no-cache \
	&& apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./

# cargo needs a dummy src/lib.rs to compile the dependencies
RUN mkdir -p src \
	&& touch src/lib.rs \
	&& cargo build --release --locked \
	&& rm -rf src

COPY . ./
RUN cargo build --release --locked --offline

RUN strip target/release/website-stalker


FROM docker.io/library/alpine:3 AS final
RUN apk upgrade --no-cache \
	&& apk add --no-cache git

COPY --from=builder /build/target/release/website-stalker /usr/local/bin/
ENTRYPOINT ["website-stalker"]

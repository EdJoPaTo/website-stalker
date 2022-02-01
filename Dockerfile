FROM docker.io/library/rust:1-alpine as builder
WORKDIR /build
RUN apk upgrade --no-cache \
    && apk add --no-cache musl-dev

# cargo needs a dummy src/main.rs to detect bin mode
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked
RUN cargo build --release --frozen --offline

# We need to touch our real main.rs file or the cached one will be used.
COPY . ./
RUN touch src/main.rs

RUN cargo build --release --frozen --offline

RUN strip target/release/website-stalker


# Start building the final image
# Use one of the cached base images https://github.com/actions/virtual-environments/blob/main/images/linux/Ubuntu2004-Readme.md#cached-docker-images
FROM docker.io/library/alpine:3.14
RUN apk upgrade --no-cache \
    && apk add --no-cache git

COPY --from=builder /build/target/release/website-stalker /usr/bin/

ENTRYPOINT ["website-stalker"]

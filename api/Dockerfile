# Step 1: Compute a recipe file.
FROM rustlang/rust:nightly-slim as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo chef prepare --recipe-path recipe.json

# Step 2: Cache project dependencies.
FROM rustlang/rust:nightly-slim as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Step 3: Build the binary
FROM rustlang/rust:nightly-slim as builder
WORKDIR /app
# Copy over the cached dependencies from above.
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release --bin api

# Step 4:
# Create a tiny output image.
# It only contains our final binary.
FROM rustlang/rust:nightly-slim as runtime
WORKDIR /app
COPY --from=builder /app/target/release/api /app
COPY Rocket.toml .
ENTRYPOINT ["./api"]

# This would be awesome, but this is will force us to install
# curl into the container. Meh.
#HEALTHCHECK --interval=10s --timeout=1s --retries=5 \
#  CMD curl http://localhost:8080/api/v1/health

# Step 5: Build test container.
FROM rustlang/rust:nightly-slim as tester
WORKDIR /app
# Copy cached files from the cacher.
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
# Insert source with dependencies for tests.
COPY src src
COPY Cargo.toml .
COPY Cargo.lock .
# Build tests.
RUN cargo build --tests
# Insert runtime config.
COPY Rocket.toml .
# Run tests.
CMD ["cargo", "test", "--", "--test-threads=1"]

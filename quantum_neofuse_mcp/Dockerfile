FROM lukemathwalker/cargo-chef:latest-rust-1.75 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin quantum_neofuse_mcp

# Runtime Stage - using Google Distroless for security and minimal size
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/quantum_neofuse_mcp /
CMD ["/quantum_neofuse_mcp"]

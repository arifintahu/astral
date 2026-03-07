# Stage 1: Build Frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app/web
COPY web/package*.json ./
RUN npm ci
COPY web/ .
RUN npm run build

# Stage 2: Build Backend
FROM rust:1.85-slim-bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Create a dummy project to cache dependencies
RUN cargo new --bin astral
WORKDIR /app/astral
COPY Cargo.toml Cargo.lock ./
# We need to add dependencies that might be platform specific or need compilation
RUN cargo build --release
RUN rm src/*.rs

# Copy actual source code
COPY src ./src
# Copy built frontend assets from previous stage
COPY --from=frontend-builder /app/web/dist ./web/dist

# Build the actual application
# We touch main.rs to ensure cargo rebuilds it
RUN touch src/main.rs
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=backend-builder /app/astral/target/release/astral .

# Create a volume for the database
VOLUME /app/data

# Expose the default port
EXPOSE 8080

# Set the entrypoint
# Note: We use the array form to ensure signals are passed correctly
CMD ["./astral"]

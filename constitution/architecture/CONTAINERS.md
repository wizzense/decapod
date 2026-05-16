# CONTAINERS.md - Container Architecture Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. Docker Fundamentals

### 1.1 Dockerfile Instructions Reference

```dockerfile
# Dockerfile instruction summary
# ============================================

# FROM - Base image selection
FROM ubuntu:22.04                          # Linux base
FROM alpine:3.18                           # Minimal Linux
FROM golang:1.21-alpine                    # Language image
FROM node:20-alpine                         # Node.js image
FROM python:3.11-slim                      # Python image
FROM eclipse-temurin:21-jre                # Java JRE
FROM --platform=linux/amd64 python:3.11    # Multi-platform
FROM scratch                                # No base (minimal)

# LABEL - Metadata
LABEL maintainer="team@example.com"
LABEL version="1.0.0"
LABEL description="Service description"
LABEL org.opencontainers.image.title="Service"
LABEL org.opencontainers.image.version="1.0"
LABEL org.opencontainers.image.source="https://github.com/example/repo"

# ARG - Build-time variables
ARG VERSION=1.0.0
ARG BUILD_DATE
ARG GIT_COMMIT
ARG REGISTRY=ghcr.io

# ENV - Environment variables (persistent in image)
ENV NODE_ENV=production
ENV APP_PORT=8080
ENV PATH="/app/bin:${PATH}"

# RUN - Execute commands during build
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir -r requirements.txt

RUN echo "deb http://repo.example.com/ stable main" > /etc/apt/sources.list.d/repo.list

# COPY - Copy files into image
COPY --chown=app:app package*.json /app/
COPY --chmod=755 ./entrypoint.sh /entrypoint.sh
COPY --from=builder /build/output /app/bin/

# ADD - Add files (supports URLs and tar extraction)
ADD https://example.com/config.tar.gz /app/config/
ADD ./app.tar.gz /app/

# WORKDIR - Set working directory
WORKDIR /app
WORKDIR /home/app

# USER - Set user for commands
USER app
USER 1000:1000

# EXPOSE - Document port (not enforced)
EXPOSE 8080 9090

# VOLUME - Define mount points
VOLUME ["/data", "/logs"]
VOLUME /var/lib/postgresql/data

# ENTRYPOINT - Container startup command (exec form - preferred)
ENTRYPOINT ["/app/entrypoint.sh"]
ENTRYPOINT ["python", "-m", "gunicorn"]

# CMD - Default arguments (overridable with docker run args)
CMD ["python", "app.py"]
CMD ["--config", "/etc/app/config.yaml"]
CMD ["serve", "--port", "8080"]

# Combined ENTRYPOINT + CMD example
ENTRYPOINT ["/entrypoint.sh"]
CMD ["--port", "8080", "--workers", "4"]

# HEALTHCHECK - Container health verification
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

HEALTHCHECK NONE  # Disable healthcheck

# ONBUILD - Triggers for child images
ONBUILD COPY package*.json /app/
ONBUILD RUN pip install --no-cache-dir -r requirements.txt

# STOPSIGNAL - Signal to stop container
STOPSIGNAL SIGTERM
STOPSIGNAL SIGKILL
```

### 1.2 Multi-Stage Build Patterns

```dockerfile
# ============================================================
# Go Application Multi-Stage Build
# ============================================================

# Stage 1: Build
FROM golang:1.21-alpine AS builder

# Install build dependencies
RUN apk add --no-cache git make gcc musl-dev

WORKDIR /build

# Copy go mod files first for better caching
COPY go.mod go.sum ./
RUN go mod download

# Copy source code
COPY . .

# Build arguments
ARG VERSION=dev
ARG GIT_COMMIT=unknown

# Build the application
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 \
    go build \
    -ldflags="-s -w -X main.Version=${VERSION} -X main.GitCommit=${GIT_COMMIT}" \
    -o /app/server \
    ./cmd/server

# Stage 2: Runtime
FROM alpine:3.18 AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    curl \
    tzdata \
    && update-ca-certificates

# Create non-root user
RUN addgroup -g 1000 -S appgroup && \
    adduser -u 1000 -S appuser -G appgroup

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/server /app/server
COPY --from=builder /build/configs /app/configs

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Set ownership
RUN chown -R appuser:appgroup /app

USER appuser

# Environment variables
ENV APP_ENV=production
ENV APP_PORT=8080

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

ENTRYPOINT ["/entrypoint.sh"]
```

```dockerfile
# ============================================================
# Node.js Application Multi-Stage Build
# ============================================================

# Stage 1: Dependencies
FROM node:20-alpine AS deps

WORKDIR /app

# Copy package files first for better caching
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production

# Stage 2: Build
FROM node:20-alpine AS builder

WORKDIR /app

# Copy dependency manifests
COPY package*.json ./

# Install all dependencies (including dev)
RUN npm ci

# Copy source code
COPY . .

# Build arguments
ARG NEXT_PUBLIC_API_URL
ARG NEXT_PUBLIC_VERSION

ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_VERSION=$NEXT_PUBLIC_VERSION

# Build the application
RUN npm run build

# Stage 3: Runtime
FROM node:20-alpine AS runtime

# Install production dependencies only
COPY --from=deps /app/node_modules ./node_modules
COPY --from=builder /app/.next /app/.next
COPY --from=builder /app/public /app/public
COPY --from=builder /app/package.json /app/package.json

# Create non-root user
RUN addgroup -g 1001 -S nextjs && \
    adduser -S nextjs -u 1001 -G nextjs

WORKDIR /app

# Set ownership
RUN chown -R nextjs:nextjs /app

USER nextjs

ENV NODE_ENV=production
ENV PORT=3000

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

CMD ["node_modules/.bin/next", "start"]
```

```dockerfile
# ============================================================
# Python Application Multi-Stage Build
# ============================================================

# Stage 1: Builder
FROM python:3.11-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Create virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:${PATH}"

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir -r requirements.txt

# Stage 2: Runtime
FROM python:3.11-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home appuser

WORKDIR /app

# Copy virtual environment from builder
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:${PATH}"

# Copy application code
COPY --chown=appuser:appuser ./src /app/src
COPY --chown=appuser:appuser ./migrations /app/migrations
COPY --chown=appuser:appuser ./config /app/config

# Switch to non-root user
USER appuser

ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONUNBUFFERED=1
ENV APP_ENV=production

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["gunicorn", "--bind", "0.0.0.0:8080", "--workers", "4", "--threads", "2", "src.app:create_app()"]
```

## 2. OCI Specifications

### 2.1 Image Manifest Specification

```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.image.manifest.v1+json",
  "config": {
    "mediaType": "application/vnd.oci.image.config.v1+json",
    "size": 7023,
    "digest": "sha256:b5b2b2c507a0944348e0303114d8d93aaaa081732b86451d9bce1f432a537bc7"
  },
  "layers": [
    {
      "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
      "size": 32654,
      "digest": "sha256:e692418f4f4d6422a474ab2aafd02b05f1ba02e46fce0ca8bb5b3dcf65a2b6c7"
    },
    {
      "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
      "size": 16724,
      "digest": "sha256:3c3a46054500ad7e2c6d6a83af9b3e1f4f1c9a6e5a9f8a7b4e3d2c1a0f9e8d7"
    }
  ],
  "annotations": {
    "org.opencontainers.image.title": "Application",
    "org.opencontainers.image.version": "1.0.0",
    "org.opencontainers.image.description": "Application description"
  }
}
```

### 2.2 Image Index for Multi-Architecture

```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.image.index.v1+json",
  "manifests": [
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "size": 7143,
      "digest": "sha256:amd64-manifest-digest",
      "platform": {
        "architecture": "amd64",
        "os": "linux",
        "os.version": "5.10",
        "variant": "v2"
      }
    },
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "size": 7143,
      "digest": "sha256:arm64-manifest-digest",
      "platform": {
        "architecture": "arm64",
        "os": "linux",
        "os.version": "5.10",
        "variant": "v8"
      }
    },
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "size": 7143,
      "digest": "sha256:armv7-manifest-digest",
      "platform": {
        "architecture": "arm",
        "os": "linux",
        "variant": "v7"
      }
    }
  ],
  "annotations": {
    "org.opencontainers.image.description": "Multi-platform image"
  }
}
```

### 2.3 Container Configuration

```json
{
  "Hostname": "container-id",
  "Domainname": "",
  "User": "appuser:appgroup",
  "AttachStdin": false,
  "AttachStdout": false,
  "AttachStderr": false,
  "Tty": false,
  "OpenStdin": false,
  "StdinOnce": false,
  "Env": [
    "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
    "NODE_ENV=production",
    "APP_PORT=8080"
  ],
  "Cmd": ["/app/server"],
  "Image": "sha256:abc123...",
  "Volumes": {
    "/data": {},
    "/logs": {}
  },
  "WorkingDir": "/app",
  "Entrypoint": ["/entrypoint.sh"],
  "Labels": {
    "maintainer": "team@example.com",
    "version": "1.0.0"
  },
  "ExposedPorts": {
    "8080/tcp": {},
    "9090/tcp": {}
  },
  "StopSignal": "SIGTERM",
  "Shell": ["/bin/sh", "-c"]
}
```

## 3. Complete Production Dockerfiles

### 3.1 Production Node.js Service Dockerfile

```dockerfile
# =============================================================================
# Node.js Production Service Dockerfile
# =============================================================================

# Build stage
FROM node:20-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    python3 \
    make \
    g++

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production=false

# Copy source code
COPY . .

# Build arguments
ARG NODE_ENV=production
ARG BUILD_VERSION=dev

ENV NODE_ENV=$NODE_ENV
ENV BUILD_VERSION=$BUILD_VERSION

# Build TypeScript
RUN npm run build

# Remove dev dependencies
RUN npm prune --production

# Production stage
FROM node:20-alpine AS production

# Install production dependencies
RUN apk add --no-cache \
    dumb-init \
    curl \
    && addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001 -G nodejs

WORKDIR /app

# Copy application from builder
COPY --from=builder --chown=nodejs:nodejs /app/dist ./dist
COPY --from=builder --chown=nodejs:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=nodejs:nodejs /app/package.json ./package.json
COPY --from=builder --chown=nodejs:nodejs /app/config ./config

# Set environment
ENV NODE_ENV=production \
    PORT=8080 \
    NPM_CONFIG_LOGLEVEL=warn \
    SENTRY_RELEASE=$BUILD_VERSION

# Create non-root user
USER nodejs

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD node -e "require('http').get('http://localhost:8080/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"

# Use dumb-init for proper signal handling
ENTRYPOINT ["dumb-init", "--"]

# Run the application
CMD ["node", "dist/main.js"]
```

### 3.2 Java Spring Boot Dockerfile

```dockerfile
# =============================================================================
# Java Spring Boot Production Dockerfile
# =============================================================================

# Build stage
FROM eclipse-temurin:21-jdk AS builder

WORKDIR /build

# Copy Maven wrapper and pom.xml
COPY mvnw .
COPY .mvn .mvn
COPY pom.xml .

# Download dependencies (layer caching)
RUN ./mvnw dependency:go-offline -B

# Copy source code
COPY src ./src

# Build arguments
ARG JAR_FILE=target/*.jar
ARG BUILD_VERSION=dev

# Build the application
RUN ./mvnw package -DskipTests -B -Dversion=$BUILD_VERSION

# Extract layers for better caching
RUN mkdir -p /build/dependency && \
    cd /build/dependency && \
    java -Djarmode=layertools -jar /build/target/*.jar extract

# Production stage
FROM eclipse-temurin:21-jre AS production

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    dumb-init \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r javagroup && useradd -r -g javagroup javauser

WORKDIR /app

# Copy extracted layers
COPY --from=builder --chown=javauser:javagroup /build/dependency/BOOT-INF/lib /appBOOT-INF/lib
COPY --from=builder --chown=javauser:javagroup /build/dependency/META-INF /app/META-INF
COPY --from=builder --chown=javauser:javagroup /build/dependency/BOOT-INF/class /app/BOOT-INF/class

# Set environment
ENV JAVA_OPTS="-Xms256m -Xmx512m -XX:+UseG1GC" \
    SPRING_PROFILES_ACTIVE=production \
    SERVER_PORT=8080

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/actuator/health || exit 1

# Use dumb-init for proper signal handling
ENTRYPOINT ["dumb-init", "--", "java", "-jar", "/app/app.jar"]
```

### 3.3 Rust Application Dockerfile

```dockerfile
# =============================================================================
# Rust Production Dockerfile
# =============================================================================

# Build stage
FROM rust:1.71-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static

WORKDIR /build

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for dependency caching
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release && \
    rm -rf src

# Copy actual source
COPY src ./src
COPY config ./config

# Build arguments
ARG GIT_COMMIT=unknown
ARG BUILD_DATE=unknown

ENV VERSION=$GIT_COMMIT

# Build release binary
RUN cargo build --release && \
    strip target/release/myapp

# Production stage
FROM alpine:3.18 AS production

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    curl \
    openssl \
    tzdata

# Create non-root user
RUN addgroup -g 1000 -S appgroup && \
    adduser -u 1000 -S appuser -G appgroup

WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=appuser:appgroup /build/target/release/myapp /app/myapp
COPY --from=builder --chown=appuser:appgroup /build/config /app/config

# Set ownership
RUN chown -R appuser:appgroup /app

USER appuser

ENV APP_ENV=production
ENV RUST_LOG=info
ENV APP_PORT=8080

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/myapp"]
```

## 4. Security Hardening

### 4.1 Security Best Practices

```dockerfile
# =============================================================================
# Security Hardened Dockerfile
# =============================================================================

# Use specific version tags, never :latest
FROM python:3.11.3-slim-bookworm

# Security: Set environment variables for security
ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    security_opt=no-new-privileges:true

# Security: Create unique application user
RUN groupadd --gid 1000 appgroup && \
    useradd --uid 1000 --gid appgroup --shell /bin/false --create-home appuser

# Security: Install only necessary packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        goss \
        && rm -rf /var/lib/apt/lists/* \
    && find /usr -name "*.pyc" -delete \
    && find /usr -name "__pycache__" -type d -delete

# Security: Add DNS resolver config
RUN echo 'nameserver 8.8.8.8' > /etc/resolv.conf

# Security: Disable services
RUN echo '#!/bin/sh\nset -e\n\nexit 0' > /usr/sbin/policy-rc.d && \
    chmod +x /usr/sbin/policy-rc.d

# Copy application with correct permissions
WORKDIR /app

COPY --chown=appuser:appgroup requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY --chown=appuser:appgroup . .

# Security: Set file permissions
RUN chmod 750 /app/config /app/keys && \
    chmod 640 /app/config/*.yaml

# Security: Switch to non-root user
USER appuser

# Security: Set working directory
WORKDIR /app

# Security: Drop capabilities
# Note: This requires Docker daemon configuration
# RUN setcap cap_drop=all /app/myapp

# Security: Use read-only filesystem (when supported)
# VOLUME ["/data", "/logs"]

# Security: No root privileges
ENV HOME=/appuser

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8080/health')" || exit 1

CMD ["python", "app.py"]
```

### 4.2 Non-Root Container Configuration

```dockerfile
# =============================================================================
# Non-root Container Example
# =============================================================================

FROM ubuntu:22.04

# Create user with specific UID/GID
RUN groupadd --gid 1000 appgroup && \
    useradd --uid 1000 --gid appgroup --shell /bin/bash --create-home appuser

# Install packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set up application
WORKDIR /app

# Create data directories
RUN mkdir -p /app/data /app/logs && \
    chown -R appuser:appgroup /app

# Copy files
COPY --chown=appuser:appgroup . .

# Switch to non-root user
USER appuser

# Verify user
RUN id

# Set default command
CMD ["/app/entrypoint.sh"]
```

### 4.3 secrets-rotation script

```bash
#!/bin/bash
# =============================================================================
# Entrypoint with Secrets Rotation
# =============================================================================

set -euo pipefail

# Source secrets from mounted secrets or environment
if [ -f /run/secrets/db_password ]; then
    export DB_PASSWORD=$(cat /run/secrets/db_password)
elif [ -n "${DB_PASSWORD:-}" ]; then
    echo "Using DB_PASSWORD from environment"
else
    echo "ERROR: No database password found"
    exit 1
fi

# Token rotation check
if [ -f /run/secrets/jwt_secret ]; then
    export JWT_SECRET=$(cat /run/secrets/jwt_secret)
fi

# Verify required secrets
for secret in DB_PASSWORD; do
    if [ -z "${!secret}" ]; then
        echo "ERROR: $secret is not set"
        exit 1
    fi
done

# Signal handling for graceful shutdown
cleanup() {
    echo "Received shutdown signal, finishing requests..."
    kill -TERM $pid
    wait $pid
    exit 0
}

trap cleanup SIGTERM SIGINT

# Start application
exec /app/server &
pid=$!

# Wait for application
wait $pid
```

## 5. Registry Patterns

### 5.1 Docker Compose with Local Registry

```yaml
# docker-compose.yml - Local development with registry
version: '3.8'

services:
  registry:
    image: registry:2.8
    ports:
      - "5000:5000"
    environment:
      REGISTRY_AUTH: htpasswd
      REGISTRY_AUTH_HTPASSWD_REALM: Registry
      REGISTRY_AUTH_HTPASSWD_PATH: /auth/htpasswd
    volumes:
      - registry-data:/var/lib/registry
      - ./auth:/auth
    restart: unless-stopped

  # Build and push service on code change
  api-build:
    image: docker:cli
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
      - ../:/workspace
    working_dir: /workspace
    command: |
      sh -c '
        docker build -t localhost:5000/api:latest ./api &&
        docker push localhost:5000/api:latest
      '
    depends_on:
      - registry
    profiles:
      - build

  # Development service pulling from local registry
  api:
    image: localhost:5000/api:latest
    ports:
      - "8080:8080"
    environment:
      - DB_HOST=postgres
      - DB_PASSWORD=devpass
    depends_on:
      postgres:
        condition: service_healthy
    restart: unless-stopped

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: app
      POSTGRES_USER: app
      POSTGRES_PASSWORD: devpass
    volumes:
      - postgres-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d app"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  registry-data:
  postgres-data:
```

### 5.2 Multi-Architecture Build Script

```bash
#!/bin/bash
# =============================================================================
# Build and Push Multi-Architecture Image
# =============================================================================

set -euo pipefail

REGISTRY="${REGISTRY:-ghcr.io}"
IMAGE_NAME="${IMAGE_NAME:-myorg/myapp}"
VERSION="${VERSION:-latest}"

# Platforms to build for
PLATFORMS="linux/amd64,linux/arm64/v8"

echo "Building multi-architecture image: ${REGISTRY}/${IMAGE_NAME}:${VERSION}"

# Login to registry (if needed)
if [[ "$REGISTRY" == *"ghcr.io"* ]]; then
    echo "$GHCR_TOKEN" | docker login ghcr.io -u "$GHCR_USERNAME" --password-stdin
fi

# Build and push using buildx
docker buildx create --name multiarch-builder --use 2>/dev/null || docker buildx use multiarch-builder
docker buildx inspect --bootstrap

# Build for multiple platforms
docker buildx build \
    --platform "$PLATFORMS" \
    --tag "${REGISTRY}/${IMAGE_NAME}:${VERSION}" \
    --tag "${REGISTRY}/${IMAGE_NAME}:latest" \
    --push \
    --builder multiarch-builder \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    --cache-from "type=registry,ref=${REGISTRY}/${IMAGE_NAME}:buildcache" \
    --cache-to "type=registry,ref=${REGISTRY}/${IMAGE_NAME}:buildcache,mode=max" \
    .

# Create and push image index
docker buildx imagetools create \
    --tag "${REGISTRY}/${IMAGE_NAME}:${VERSION}" \
    --tag "${REGISTRY}/${IMAGE_NAME}:latest" \
    "${REGISTRY}/${IMAGE_NAME}:linux-amd64" \
    "${REGISTRY}/${IMAGE_NAME}:linux-arm64"

echo "Successfully built and pushed multi-architecture image"

# Verify manifest
docker buildx imagetools inspect "${REGISTRY}/${IMAGE_NAME}:${VERSION}"
```

### 5.3 Image Promotion Workflow

```yaml
# .github/workflows/image-promotion.yml
name: Image Promotion

on:
  workflow_dispatch:
    inputs:
      source_tag:
        description: 'Source image tag'
        required: true
      target_tag:
        description: 'Target image tag'
        required: true

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  promote:
    runs-on: ubuntu-latest
    permissions:
      packages: write
    
    steps:
      - name: Login to Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Pull source image
        run: |
          docker pull ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.source_tag }}
          docker pull ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.source_tag }}-linux-amd64
          docker pull ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.source_tag }}-linux-arm64
      
      - name: Retag images
        run: |
          docker tag ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.source_tag }} \
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}
          docker tag ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.source_tag }}-linux-amd64 \
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}-linux-amd64
          docker tag ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.source_tag }}-linux-arm64 \
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}-linux-arm64
      
      - name: Push promoted images
        run: |
          docker push ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}
          docker push ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}-linux-amd64
          docker push ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}-linux-arm64
      
      - name: Create and push manifest
        run: |
          docker manifest create \
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }} \
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}-linux-amd64 \
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}-linux-arm64
          docker manifest push ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ inputs.target_tag }}
```

## 6. Complete Docker Compose for Production

### 6.1 Production Stack Example

```yaml
# docker-compose.production.yml
version: '3.8'

services:
  api:
    build:
      context: ./api
      dockerfile: Dockerfile
      target: production
      args:
        - BUILD_VERSION=${GIT_SHA:-dev}
    image: ${REGISTRY:-ghcr.io}/myorg/api:${IMAGE_TAG:-latest}
    container_name: api
    restart: unless-stopped
    ports:
      - "127.0.0.1:8080:8080"
    environment:
      - NODE_ENV=production
      - APP_PORT=8080
      - DB_HOST=postgres
      - DB_PORT=5432
      - DB_NAME=app
      - DB_USER=app
      - DB_PASSWORD_FILE=/run/secrets/db_password
      - REDIS_HOST=redis
      - REDIS_PORT=6379
      - REDIS_PASSWORD_FILE=/run/secrets/redis_password
    secrets:
      - db_password
      - redis_password
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    healthcheck:
      test: ["CMD", "node", "-e", "require('http').get('http://localhost:8080/health', (r) => process.exit(r.statusCode === 200 ? 0 : 1))"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '0.5'
          memory: 512M
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
    networks:
      - backend

  worker:
    image: ${REGISTRY:-ghcr.io}/myorg/api:${IMAGE_TAG:-latest}
    container_name: worker
    restart: unless-stopped
    command: ["node", "dist/worker.js"]
    environment:
      - NODE_ENV=production
      - DB_HOST=postgres
      - DB_PORT=5432
      - DB_NAME=app
      - DB_USER=app
      - DB_PASSWORD_FILE=/run/secrets/db_password
      - REDIS_HOST=redis
      - REDIS_PORT=6379
      - REDIS_PASSWORD_FILE=/run/secrets/redis_password
    secrets:
      - db_password
      - redis_password
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    deploy:
      replicas: 2
      resources:
        limits:
          cpus: '1'
          memory: 1G
        reservations:
          cpus: '0.25'
          memory: 256M
    logging:
      driver: "json-file"
      options:
        max-size: "50m"
        max-file: "3"
    networks:
      - backend

  postgres:
    image: postgres:15-alpine
    container_name: postgres
    restart: unless-stopped
    ports:
      - "127.0.0.1:5432:5432"
    environment:
      POSTGRES_DB: app
      POSTGRES_USER: app
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    secrets:
      - db_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U app -d app"]
      interval: 10s
      timeout: 5s
      retries: 5
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
    networks:
      - backend

  redis:
    image: redis:7-alpine
    container_name: redis
    restart: unless-stopped
    ports:
      - "127.0.0.1:6379:6379"
    command: redis-server --requirepass-file /run/secrets/redis_password --appendonly yes
    secrets:
      - redis_password
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "-a", "$(cat /run/secrets/redis_password)", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 1G
    logging:
      driver: "json-file"
      options:
        max-size: "50m"
        max-file: "3"
    networks:
      - backend

  nginx:
    image: nginx:1.25-alpine
    container_name: nginx
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/conf.d:/etc/nginx/conf.d:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
      - nginx_cache:/var/cache/nginx
      - nginx_logs:/var/log/nginx
    depends_on:
      - api
    healthcheck:
      test: ["CMD", "nginx", "-t"]
      interval: 30s
      timeout: 10s
      retries: 3
    logging:
      driver: "json-file"
      options:
        max-size: "50m"
        max-file: "5"
    networks:
      - backend

  # Monitoring stack
  prometheus:
    image: prom/prometheus:v2.47.0
    container_name: prometheus
    restart: unless-stopped
    ports:
      - "127.0.0.1:9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=15d'
      - '--web.enable-lifecycle'
    networks:
      - backend

  grafana:
    image: grafana/grafana:10.1.0
    container_name: grafana
    restart: unless-stopped
    ports:
      - "127.0.0.1:3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD_FILE=/run/secrets/grafana_password
      - GF_USERS_ALLOW_SIGN_UP=false
      - GF_SERVER_ROOT_URL=https://grafana.example.com
    secrets:
      - grafana_password
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning:ro
    depends_on:
      - prometheus
    networks:
      - backend

volumes:
  postgres_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/postgres-data
  redis_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /mnt/redis-data
  prometheus_data:
  grafana_data:
  nginx_cache:
  nginx_logs:

networks:
  backend:
    driver: bridge
    ipam:
      config:
        - subnet: 172.28.0.0/16

secrets:
  db_password:
    file: ./secrets/db_password.txt
  redis_password:
    file: ./secrets/redis_password.txt
  grafana_password:
    file: ./secrets/grafana_password.txt
```

### 6.2 Nginx Configuration

```nginx
# nginx/nginx.conf
worker_processes auto;
worker_rlimit_nofile 65535;

events {
    worker_connections 4096;
    use epoll;
    multi_accept on;
}

http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    # Hide nginx version
    server_tokens off;

    # Logging
    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for" '
                    'rt=$request_time uct="$upstream_connect_time" '
                    'uht="$upstream_header_time" urt="$upstream_response_time"';

    access_log /var/log/nginx/access.log main buffer=16k flush=2s;
    error_log /var/log/nginx/error.log warn;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';" always;

    # Performance
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    keepalive_requests 1000;
    types_hash_max_size 2048;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml application/json application/javascript 
               application/xml application/xml+rss text/javascript application/x-javascript
               application/wasm application/vnd.ms-fontobject application/x-font-ttf font/opentype;
    gzip_min_length 256;
    gzip_disable "msie6";

    # Rate limiting zones
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/s;
    limit_req_zone $binary_remote_addr zone=auth:10m rate=10r/s;
    limit_conn_zone $binary_remote_addr zone=addr:10m;

    # Upstream definitions
    upstream api_backend {
        zone api_backend 64k;
        least_conn;
        
        server api:8080 max_fails=3 fail_timeout=30s;
        
        keepalive 32;
    }

    # HTTP server (redirect to HTTPS)
    server {
        listen 80;
        listen [::]:80;
        server_name _;
        
        location /.well-known/acme-challenge/ {
            root /var/www/certbot;
        }
        
        location / {
            return 301 https://$host$request_uri;
        }
    }

    # HTTPS server
    server {
        listen 443 ssl http2;
        listen [::]:443 ssl http2;
        server_name _;

        # SSL configuration
        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;
        ssl_trusted_certificate /etc/nginx/ssl/chain.pem;
        ssl_session_timeout 1d;
        ssl_session_cache shared:SSL:50m;
        ssl_session_tickets off;
        
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384;
        ssl_prefer_server_ciphers off;
        
        ssl_stapling on;
        ssl_stapling_verify on;

        # Security headers
        add_header Strict-Transport-Security "max-age=63072000" always;

        # API endpoints
        location /api/ {
            limit_req zone=api burst=50 nodelay;
            limit_conn addr 50;

            proxy_pass http://api_backend;
            proxy_http_version 1.1;
            
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Request-ID $request_id;
            
            proxy_connect_timeout 10s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
            
            proxy_buffering on;
            proxy_buffer_size 4k;
            proxy_buffers 8 16k;
            proxy_busy_buffers_size 24k;
            
            add_header X-Upstream-Status $upstream_status;
            add_header X-Upstream-Response-Time $upstream_response_time;
        }

        # Auth endpoints with stricter limits
        location /api/auth/ {
            limit_req zone=auth burst=5 nodelay;

            proxy_pass http://api_backend;
            proxy_http_version 1.1;
            
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # WebSocket support
        location /ws/ {
            proxy_pass http://api_backend;
            proxy_http_version 1.1;
            
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            
            proxy_read_timeout 86400;
            proxy_send_timeout 86400;
        }

        # Health check endpoint
        location /health {
            access_log off;
            proxy_pass http://api_backend;
            proxy_http_version 1.1;
            
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            
            proxy_connect_timeout 5s;
            proxy_read_timeout 5s;
        }

        # Metrics endpoint (internal only)
        location /metrics {
            internal;
            proxy_pass http://prometheus:9090;
            proxy_http_version 1.1;
        }

        # Static content
        location /static/ {
            alias /var/www/static/;
            expires 1y;
            add_header Cache-Control "public, immutable";
            
            # Enable CORS for static assets
            add_header Access-Control-Allow-Origin "*";
            add_header Access-Control-Allow-Methods "GET";
        }

        # Health check for load balancer
        location /nginx-health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
    }
}
```

## 7. Decision Matrices

### 7.1 Base Image Selection Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              Base Image Selection Matrix                                 │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Image Type              │ Pros                           │ Cons                        │
├─────────────────────────┼────────────────────────────────┼─────────────────────────────┤
│ Alpine                  │ Small (5MB), fast to pull     │ Not all packages available  │
│                         │ Minimal attack surface        │ Musl vs glibc issues        │
├─────────────────────────┼────────────────────────────────┼─────────────────────────────┤
│ Debian Slim            │ Full package compatibility    │ Larger size (~80MB)         │
│                         │ Stable, well-tested           │ More updates to manage      │
├─────────────────────────┼────────────────────────────────┼─────────────────────────────┤
│ Ubuntu                 │ Full Ubuntu ecosystem         │ Large size (77MB+)          │
│                         │ Familiar for Ubuntu users     │ More frequent updates       │
├─────────────────────────┼────────────────────────────────┼─────────────────────────────┤
│ distroless             │ Minimal (25MB), no shell      │ Debugging more difficult    │
│                         │ Security focused              │ No package manager          │
├─────────────────────────┼────────────────────────────────┼─────────────────────────────┤
│ scratch                │ Minimal possible (just binary) │ No OS, no debugging         │
│                         │ Maximum security              │ Must handle all signals     │
├─────────────────────────┼────────────────────────────────┼─────────────────────────────┤
│ Distroless static      │ Tiny, no shell, static binary │ Limited use case            │
│                         │ Very secure                   │ For Go/Rust only            │
├─────────────────────────┼────────────────────────────────┼─────────────────────────────┤
│ Language-specific      │ Pre-configured for language   │ Larger than minimal         │
│                         │ Better caching                │ May include unnecessary     │
└─────────────────────────┴────────────────────────────────┴─────────────────────────────┘
```

### 7.2 Build Strategy Decision Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              Build Strategy Decision Matrix                             │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Scenario                       │ Recommended Strategy                 │ Notes         │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ Go/Rust/C binaries            │ Multi-stage, scratch or distroless  │ Static binary  │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ Node.js apps                  │ Multi-stage, node base               │ Build in deps  │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ Python apps                   │ Multi-stage, venv + slim             │ Compile deps  │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ Java/JVM apps                 │ Multi-stage, layertools extract      │ Better caching │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ Large monorepo                │ BuildKit cache mounts                │ Share cache    │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ Multiple services             │ Shared base image + service images   │ Layer sharing │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ Frequent small updates        │ BuildKit inline cache                │ Incremental   │
├───────────────────────────────┼─────────────────────────────────────┼────────────────┤
│ CI/CD with caching            │ External cache to registry          │ Multi-stage   │
└───────────────────────────────┴─────────────────────────────────────┴────────────────┘
```

## 8. Anti-Patterns

### 8.1 Common Docker Anti-Patterns

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            Docker Anti-Patterns to Avoid                                 │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Using :latest tag              │ Unpredictable builds           │ Use specific versions  │
│                                 │ No rollback possible           │ or SHA digests         │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Not using .dockerignore        │ Large images, secrets exposed  │ Create .dockerignore  │
│                                 │ Slow builds                    │ with exclusions        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Running as root                │ Security vulnerability          │ Create and use        │
│                                 │ Container escape risks         │ non-root user         │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Missing health checks         │ No auto-restart on failure     │ Add HEALTHCHECK       │
│                                 │ Kubernetes won't detect death │ directive             │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ COPY everything                │ Large images, cache invalidation│ Use .dockerignore    │
│                                 │ Secrets in image               │ Copy specific files   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No multi-stage builds         │ Large final images             │ Separate build and   │
│                                 │ Build tools in production      │ runtime stages       │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ apt-get without cleanup        │ Large image size               │ rm -rf /var/lib/apt  │
│                                 │ Unnecessary cache              │ lists/* in same RUN  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Multiple FROM statements       │ Confusing, potential misuse    │ Use AS to name stages │
│ without naming                 │                                 │                        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ CMD args not as array          │ Unexpected shell behavior      │ Use exec form         │
│                                 │ Signal handling issues         │ CMD ["arg1", "arg2"] │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No signal proxy               │ Graceful shutdown doesn't work  │ Use dumb-init or      │
│                                 │ Force kill after 10s           │ exec with trap       │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ ENV after COPY                │ Cache invalidation              │ Put ENV before COPY  │
│                                 │ Inconsistent builds            │ for better caching   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No resource limits            │ Noisy neighbor issues           │ Set memory/CPU limits │
│                                 │ OOM kills                       │ in docker-compose    │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Debug ports exposed           │ Security risk                  │ Use 127.0.0.1 binding │
│                                 │ Unintended access               │ for debug ports      │
└─────────────────────────────────┴───────────────────────────────┴────────────────────────┘
```

### 8.2 Bad vs Good Examples

```dockerfile
# BAD: Multiple bad practices
FROM ubuntu:latest

RUN apt-get update && apt-get install -y curl python nodejs

COPY . /app

WORKDIR /app

RUN pip install -r requirements.txt

RUN useradd -m appuser

USER root
# Running as root!

CMD python app.py

# GOOD: Security-hardened multi-stage build
FROM python:3.11-slim AS builder

WORKDIR /build

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

FROM python:3.11-slim AS production

RUN groupadd -g 1000 appgroup && \
    useradd -u 1000 -g appgroup --shell /bin/false --create-home appuser

WORKDIR /app

COPY --from=builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --chown=appuser:appgroup . .

USER appuser

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8080/health')" || exit 1

CMD ["python", "app.py"]
```

## 9. Testing Containers

### 9.1 Container Testing with Goss

```yaml
# tests/goss.yaml - Container validation
# Install: dgoss run -it image

package:
  curl:
    installed: true
  ca-certificates:
    installed: true

file:
  /app:
    exists: true
    mode: "0755"
    owner: appuser
    group: appgroup
  /app/config:
    exists: true
    mode: "0750"
  /app/server:
    exists: true
    mode: "0755"
  /etc/resolv.conf:
    exists: true
    contains:
      - "8.8.8.8"

user:
  appuser:
    exists: true
    uid: 1000
    gid: 1000
    home: /home/appuser
    shell: /bin/false

group:
  appgroup:
    exists: true
    gid: 1000

process:
  server:
    running: true
    count: 1

http:
  http://localhost:8080/health:
    status: 200
    timeout: 5000
    body:
      - "healthy"

command:
  python --version:
    exit-status: 0
    stdout:
      - "^3.11"
```

### 9.2 Docker Security Scanning

```bash
#!/bin/bash
# =============================================================================
# Container Security Scan Script
# =============================================================================

set -euo pipefail

IMAGE="$1"
TRIVY_DB_DIR="${TRIVY_DB_DIR:-/tmp/trivy-db}"

echo "=== Scanning $IMAGE for vulnerabilities ==="

# Run Trivy vulnerability scanner
trivy image \
    --severity HIGH,CRITICAL \
    --ignore-unfixed \
    --cache-dir "$TRIVY_DB_DIR" \
    --format json \
    --output /tmp/scan-results.json \
    "$IMAGE"

# Parse results
CRITICAL=$(jq '[.Results[] | select(.Vulnerabilities != null) | .Vulnerabilities[] | select(.Severity == "CRITICAL")] | length' /tmp/scan-results.json)
HIGH=$(jq '[.Results[] | select(.Vulnerabilities != null) | .Vulnerabilities[] | select(.Severity == "HIGH")] | length' /tmp/scan-results.json)

echo "Critical vulnerabilities: $CRITICAL"
echo "High vulnerabilities: $HIGH"

# Fail on critical vulnerabilities
if [ "$CRITICAL" -gt 0 ]; then
    echo "FAILED: Found $CRITICAL critical vulnerabilities"
    exit 1
fi

if [ "$HIGH" -gt 10 ]; then
    echo "WARNING: Found $HIGH high vulnerabilities"
fi

echo "Scan completed successfully"
```

---

## Links

### Official Documentation
- [Docker Documentation](https://docs.docker.com/)
- [Dockerfile Reference](https://docs.docker.com/engine/reference/builder/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)
- [Best Practices for Writing Dockerfiles](https://docs.docker.com/develop/develop-images/dockerfile_best-practices/)

### OCI Specifications
- [OCI Image Format Specification](https://github.com/opencontainers/image-spec)
- [OCI Runtime Specification](https://github.com/opencontainers/runtime-spec)
- [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec)

### Security
- [Docker Security](https://docs.docker.com/engine/security/)
- [Snyk Docker Security](https://docs.snyk.io/products/snyk-container/)
- [Trivy Scanner](https://aquasecurity.github.io/trivy/)
- [Dockle](https://good-with-usability.gluo.io/dockle/)

### Registry & Distribution
- [Docker Hub](https://hub.docker.com/)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [Google Container Registry](https://cloud.google.com/container-registry)
- [Amazon ECR](https://docs.aws.amazon.com/ecr/)

### Multi-Architecture
- [Docker BuildX](https://github.com/docker/buildx)
- [Manifest Tool](https://github.com/estesp/manifest-tool)
- [Multi-arch builds](https://www.docker.com/blog/multi-platform-docker-builds/)

### Testing
- [Goss](https://github.com/aelsabbahy/goss)
- [Container Structure Test](https://github.com/GoogleContainerTools/container-structure-test)
- [Hadolint](https://github.com/hadolint/hadolint)

### Tools
- [BuildKit](https://github.com/moby/buildkit)
- [Docker Compose](https://github.com/docker/compose)
- [Skopeo](https://github.com/containers/skopeo)
- [Podman](https://github.com/containers/podman)
- [Kaniko](https://github.com/GoogleContainerTools/kaniko)

### Best Practices
- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker)
- [NIST Container Security Guide](https://nvd.nist.gov/general/cspecial-publication)
- [Snyk Dockerfile Best Practices](https://snyk.io/blog/10-best-practices-to-build-docker-images-with-security/)
# syntax=docker/dockerfile:1.7
#
# Frontend image — static bundle built with Vite, served by nginx.
#
# Build context = repo root (see compose.yaml `context: ..`). We lay out
# the in-container paths to mirror the host (`/app/frontend`,
# `/app/docs/contracts/openapi.json`) so `npm run gen:api` — which uses
# the relative path `../docs/contracts/openapi.json` — works unchanged.

# ---------------------- builder ----------------------
FROM node:20-alpine AS builder

WORKDIR /app/frontend

# Install deps first (with the lockfile if present) for cache friendliness.
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci --no-audit --no-fund

# Bring in the OpenAPI contract at the host-relative path.
COPY docs/contracts/openapi.json /app/docs/contracts/openapi.json

# Now the rest of the frontend sources.
COPY frontend/. .

# Regenerate types from the contract, then build.
RUN npm run gen:api && npm run build

# ---------------------- runtime ----------------------
FROM nginx:1.27-alpine AS runtime

# Small SPA-fallback nginx config inline (proper nginx.conf with
# security headers is DevOps-phase work).
RUN printf 'server {\n\
    listen 80;\n\
    listen [::]:80;\n\
    server_name _;\n\
    root /usr/share/nginx/html;\n\
    index index.html;\n\
\n\
    # Hashed static assets → cache aggressively.\n\
    location ~* \\.(?:js|css|woff2?|png|jpg|jpeg|svg|webp|ico)$ {\n\
        expires 7d;\n\
        add_header Cache-Control "public, max-age=604800, immutable";\n\
        try_files $uri =404;\n\
    }\n\
\n\
    # SPA fallback.\n\
    location / {\n\
        try_files $uri $uri/ /index.html;\n\
        add_header Cache-Control "no-cache";\n\
    }\n\
}\n' > /etc/nginx/conf.d/default.conf

COPY --from=builder /app/frontend/dist /usr/share/nginx/html

EXPOSE 80

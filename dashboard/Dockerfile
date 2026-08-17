# ── Dependencies ─────────────────────────────────────────────────────────
FROM node:22-alpine AS deps
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci

# ── Builder ──────────────────────────────────────────────────────────────
FROM node:22-alpine AS builder
WORKDIR /app
# NEXT_PUBLIC_* vars are inlined at build time — pass them here, not at runtime.
ARG NEXT_PUBLIC_ENGINE_URL=http://localhost:8080
ENV NEXT_PUBLIC_ENGINE_URL=$NEXT_PUBLIC_ENGINE_URL
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

# ── Runner ───────────────────────────────────────────────────────────────
FROM node:22-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production
ENV PORT=3000

COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static

EXPOSE 3000
CMD ["node", "server.js"]

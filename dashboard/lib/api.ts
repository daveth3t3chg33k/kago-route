/**
 * Typed client for the KagoRoute engine's read endpoints.
 *
 * The engine URL is injected at build time via NEXT_PUBLIC_ENGINE_URL
 * (see .env.local.example and the Dockerfile build ARG).
 */

import type { FlowDocument } from "./schema/types";

export const ENGINE_URL = (
  process.env.NEXT_PUBLIC_ENGINE_URL ?? "http://localhost:8080"
).replace(/\/+$/, "");

export interface EngineHealth {
  status: string;
  service: string;
  version: string;
  uptime_secs: number;
  session_store: "memory" | "redis";
  database: "ok" | "unavailable" | "not-configured";
  flow: {
    id: string;
    version: number;
  };
}

export interface FlowInfo {
  id: string;
  name: string;
  description: string;
  version: number;
  start: string;
  timeouts: {
    session: number;
    step: number;
  };
  nodes: string[];
}

export interface ActiveSession {
  id: string;
  vars: Record<string, unknown> | null;
}

export interface SessionsResponse {
  count: number;
  sessions: ActiveSession[];
}

export class EngineError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message);
    this.name = "EngineError";
  }
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  const res = await fetch(`${ENGINE_URL}${path}`, {
    signal,
    // Don't let the browser cache engine state.
    cache: "no-store",
  });
  if (!res.ok) {
    throw new EngineError(`GET ${path} failed with ${res.status}`, res.status);
  }
  return (await res.json()) as T;
}

export const engineApi = {
  health(signal?: AbortSignal) {
    return getJson<EngineHealth>("/health", signal);
  },
  flow(signal?: AbortSignal) {
    return getJson<FlowInfo>("/flow", signal);
  },
  sessions(signal?: AbortSignal) {
    return getJson<SessionsResponse>("/sessions", signal);
  },
  flowSchema(signal?: AbortSignal) {
    return getJson<FlowDocument>("/flow/schema", signal);
  },
};

/** Format an uptime in seconds as a compact human string. */
export function formatUptime(secs: number): string {
  if (secs < 60) return `${secs}s`;
  const minutes = Math.floor(secs / 60);
  if (minutes < 60) return `${minutes}m ${secs % 60}s`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}


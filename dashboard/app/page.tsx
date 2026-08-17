"use client";

import { useCallback, useEffect, useState } from "react";
import {
  engineApi,
  formatUptime,
  type EngineHealth,
  type FlowInfo,
  type SessionsResponse,
} from "@/lib/api";
import { Sidebar } from "@/components/Sidebar";

const POLL_MS = 5000;

// ── Polling hook ────────────────────────────────────────────────────────

function useEngineData() {
  const [health, setHealth] = useState<EngineHealth | null>(null);
  const [flow, setFlow] = useState<FlowInfo | null>(null);
  const [sessions, setSessions] = useState<SessionsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  const refresh = useCallback(async (signal?: AbortSignal) => {
    try {
      const [h, f, s] = await Promise.all([
        engineApi.health(signal),
        engineApi.flow(signal),
        engineApi.sessions(signal),
      ]);
      setHealth(h);
      setFlow(f);
      setSessions(s);
      setError(null);
      setLastUpdated(new Date());
    } catch (e) {
      if ((e as Error).name !== "AbortError") {
        setError((e as Error).message);
        // The engine is unreachable — drop the stale health so the status
        // indicators flip to offline instead of showing a green "online".
        setHealth(null);
      }
    }
  }, []);

  useEffect(() => {
    let stopped = false;
    let ctrl: AbortController | null = null;
    let timer: ReturnType<typeof setTimeout> | undefined;

    // Chain polls with setTimeout after each completion, so requests never
    // overlap and an older slow response can't clobber fresher state.
    const tick = async () => {
      ctrl?.abort();
      ctrl = new AbortController();
      await refresh(ctrl.signal);
      if (!stopped) timer = setTimeout(tick, POLL_MS);
    };

    void tick();
    return () => {
      stopped = true;
      ctrl?.abort();
      if (timer) clearTimeout(timer);
    };
  }, [refresh]);

  return { health, flow, sessions, error, lastUpdated, refresh };
}

// ── Small presentational pieces ─────────────────────────────────────────

function StatusDot({ online }: { online: boolean }) {
  return (
    <span className="relative flex h-2 w-2">
      {online ? (
        <>
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
          <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-400" />
        </>
      ) : (
        <span className="relative inline-flex h-2 w-2 rounded-full bg-red-400" />
      )}
    </span>
  );
}

function StatCard({
  label,
  value,
  sub,
  tone = "default",
}: {
  label: string;
  value: string;
  sub?: string;
  tone?: "default" | "ok" | "bad" | "warn";
}) {
  const valueColor =
    tone === "ok"
      ? "text-emerald-300"
      : tone === "bad"
        ? "text-red-300"
        : tone === "warn"
          ? "text-amber-300"
          : "text-white";
  return (
    <div className="rounded-2xl border border-white/5 bg-white/[0.03] p-5 transition-colors hover:border-white/10">
      <dt className="text-xs font-medium uppercase tracking-wider text-zinc-500">
        {label}
      </dt>
      <dd className={`mt-2 text-xl font-semibold tracking-tight ${valueColor}`}>
        {value}
      </dd>
      {sub ? <dd className="mt-1 text-xs text-zinc-500">{sub}</dd> : null}
    </div>
  );
}

function VarsChips({ vars }: { vars: Record<string, unknown> | null }) {
  if (!vars) return <span className="text-xs text-zinc-600">—</span>;
  const entries = Object.entries(vars).slice(0, 4);
  return (
    <div className="flex flex-wrap gap-1.5">
      {entries.map(([k, v]) => (
        <span
          key={k}
          className="rounded-md border border-white/10 bg-white/5 px-1.5 py-0.5 font-mono text-[11px] text-zinc-300"
        >
          {k}=<span className="text-emerald-300">{String(v)}</span>
        </span>
      ))}
      {Object.keys(vars).length > 4 ? (
        <span className="px-1 py-0.5 font-mono text-[11px] text-zinc-600">
          +{Object.keys(vars).length - 4}
        </span>
      ) : null}
    </div>
  );
}

function SessionsTable({ sessions }: { sessions: SessionsResponse | null }) {
  const count = sessions?.count ?? 0;
  return (
    <section className="rounded-2xl border border-white/10 bg-[#0b1210]">
      <div className="flex items-center justify-between border-b border-white/5 px-5 py-4">
        <span className="flex items-center gap-2 text-sm font-medium text-white">
          <StatusDot online={count > 0} />
          Live sessions
        </span>
        <span className="rounded-full border border-white/10 px-2 py-0.5 font-mono text-xs text-zinc-400">
          {count} active
        </span>
      </div>

      {count === 0 ? (
        <div className="px-5 py-12 text-center">
          <p className="text-sm font-medium text-zinc-300">No live sessions</p>
          <p className="mt-1 font-mono text-xs text-zinc-500">
            Dial *483*42# and walk a flow — sessions appear here in real time.
          </p>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm">
            <thead>
              <tr className="border-b border-white/5 text-xs uppercase tracking-wide text-zinc-500">
                <th className="px-5 py-3 font-medium">Session</th>
                <th className="px-5 py-3 font-medium">Variables</th>
              </tr>
            </thead>
            <tbody>
              {sessions?.sessions.map((s) => (
                <tr
                  key={s.id}
                  className="border-b border-white/5 last:border-0 hover:bg-white/[0.03]"
                >
                  <td className="px-5 py-3.5">
                    <span className="inline-flex items-center gap-1.5 font-mono text-xs text-emerald-300">
                      <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-400" />
                      {s.id}
                    </span>
                  </td>
                  <td className="px-5 py-3.5">
                    <VarsChips vars={s.vars} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

function FlowPanel({ flow }: { flow: FlowInfo | null }) {
  return (
    <section className="rounded-2xl border border-white/10 bg-[#0b1210]">
      <div className="flex items-center justify-between border-b border-white/5 px-5 py-4">
        <span className="text-sm font-medium text-white">Loaded flow</span>
        {flow ? (
          <span className="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 font-mono text-xs text-emerald-300">
            v{flow.version}
          </span>
        ) : null}
      </div>

      {!flow ? (
        <div className="px-5 py-12 text-center">
          <p className="text-sm text-zinc-500">Flow metadata unavailable.</p>
        </div>
      ) : (
        <dl className="space-y-4 px-5 py-4 text-sm">
          <div>
            <dt className="text-xs uppercase tracking-wider text-zinc-500">
              Name
            </dt>
            <dd className="mt-0.5 font-medium text-white">{flow.name}</dd>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wider text-zinc-500">
              Description
            </dt>
            <dd className="mt-0.5 text-zinc-400">{flow.description}</dd>
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <dt className="text-xs uppercase tracking-wider text-zinc-500">
                Start node
              </dt>
              <dd className="mt-0.5 font-mono text-emerald-300">{flow.start}</dd>
            </div>
            <div>
              <dt className="text-xs uppercase tracking-wider text-zinc-500">
                Timeouts
              </dt>
              <dd className="mt-0.5 font-mono text-zinc-300">
                session {flow.timeouts.session}s · step {flow.timeouts.step}s
              </dd>
            </div>
          </div>
          <div>
            <dt className="text-xs uppercase tracking-wider text-zinc-500">
              Nodes ({flow.nodes.length})
            </dt>
            <dd className="mt-1.5 flex flex-wrap gap-1.5">
              {flow.nodes.map((n) => (
                <span
                  key={n}
                  className="rounded-md border border-white/10 bg-white/5 px-1.5 py-0.5 font-mono text-[11px] text-zinc-400"
                >
                  {n}
                </span>
              ))}
            </dd>
          </div>
        </dl>
      )}
    </section>
  );
}

// ── Page ────────────────────────────────────────────────────────────────

export default function Dashboard() {
  const { health, flow, sessions, error, lastUpdated, refresh } =
    useEngineData();
  const online = health?.status === "ok";
  const lastUpdatedLabel = lastUpdated?.toLocaleTimeString() ?? "…";

  return (
    <div className="flex min-h-screen">
      <Sidebar />

      <main className="min-w-0 flex-1">
        {/* Topbar */}
        <header className="sticky top-0 z-40 border-b border-white/5 bg-[#060a09]/80 backdrop-blur-md">
          <div className="flex items-center justify-between px-6 py-4">
            <div>
              <h1 className="text-lg font-semibold tracking-tight text-white">
                Overview
              </h1>
              <p className="text-xs text-zinc-500">
                Engine status · live sessions · loaded flow
              </p>
            </div>

            <div className="flex items-center gap-4">
              <div className="hidden items-center gap-2 text-xs text-zinc-400 sm:flex">
                <StatusDot online={online} />
                <span>{online ? "engine online" : "engine offline"}</span>
                <span className="text-zinc-600">·</span>
                <span className="tabular-nums">updated {lastUpdatedLabel}</span>
              </div>
              <button
                onClick={() => void refresh()}
                className="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:border-emerald-500/40 hover:bg-emerald-500/10"
              >
                Refresh
              </button>
            </div>
          </div>
        </header>

        <div className="mx-auto max-w-6xl space-y-6 px-6 py-6">
          {/* Offline banner */}
          {error ? (
            <div className="flex items-center justify-between gap-4 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
              <span className="truncate">
                <span className="font-medium">Engine unreachable:</span>{" "}
                <span className="font-mono text-xs">{error}</span>
              </span>
              <button
                onClick={() => void refresh()}
                className="shrink-0 rounded-lg border border-red-400/30 px-2.5 py-1 text-xs font-medium text-red-100 transition-colors hover:bg-red-500/20"
              >
                Retry
              </button>
            </div>
          ) : null}

          {/* Stat cards */}
          <dl className="grid grid-cols-2 gap-4 sm:grid-cols-3 xl:grid-cols-6">
            <StatCard
              label="Engine"
              value={online ? "online" : "offline"}
              sub={health ? `v${health.version}` : "no response"}
              tone={online ? "ok" : "bad"}
            />
            <StatCard
              label="Uptime"
              value={health ? formatUptime(health.uptime_secs) : "—"}
              sub="since boot"
            />
            <StatCard
              label="Session store"
              value={health?.session_store ?? "—"}
              sub={health?.session_store === "redis" ? "Redis" : "in-memory"}
              tone={health?.session_store === "redis" ? "ok" : "warn"}
            />
            <StatCard
              label="Database"
              value={health?.database ?? "—"}
              tone={
                !health
                  ? "default"
                  : health.database === "ok"
                    ? "ok"
                    : health.database === "not-configured"
                      ? "warn"
                      : "bad"
              }
            />
            <StatCard
              label="Flow"
              value={health ? `${health.flow.id} v${health.flow.version}` : "—"}
              sub={flow?.name}
            />
            <StatCard
              label="Live sessions"
              value={String(sessions?.count ?? "—")}
              sub={sessions && sessions.count > 0 ? "active now" : "idle"}
              tone={sessions && sessions.count > 0 ? "ok" : "default"}
            />
          </dl>

          {/* Panels */}
          <div className="grid gap-6 xl:grid-cols-5">
            <div className="xl:col-span-3">
              <SessionsTable sessions={sessions} />
            </div>
            <div className="xl:col-span-2">
              <FlowPanel flow={flow} />
            </div>
          </div>

          {/* Mobile-only engine strip */}
          <div className="flex items-center gap-2 rounded-xl border border-white/5 bg-white/[0.02] px-4 py-3 text-xs text-zinc-500 sm:hidden">
            <StatusDot online={online} />
            <span>{online ? "engine online" : "engine offline"}</span>
            <span className="text-zinc-600">·</span>
            <span>updated {lastUpdatedLabel}</span>
          </div>
        </div>
      </main>
    </div>
  );
}

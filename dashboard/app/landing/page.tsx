import Link from "next/link";
import type { JSX } from "react";

const NAV_LINKS = ["Product", "Docs", "Pricing"];

const FEATURES: {
  title: string;
  description: string;
  icon: JSX.Element;
}[] = [
  {
    title: "Schema-driven menus",
    description:
      "Publish a JSON or YAML menu tree. KagoRoute walks it, validates inputs, and replies CON/END — no telco state machines by hand.",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6">
        <path d="M4 6h16M4 12h16M4 18h10" />
        <circle cx="19" cy="18" r="2" />
      </svg>
    ),
  },
  {
    title: "Unified ledger",
    description:
      "Feature-phone orders land in the same PostgreSQL database as your smartphone app. No fragmented records, no reconciliation pain.",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6">
        <path d="M3 12h18M3 6h18M3 18h18" />
        <circle cx="7" cy="6" r="1.2" fill="currentColor" />
        <circle cx="7" cy="12" r="1.2" fill="currentColor" />
        <circle cx="7" cy="18" r="1.2" fill="currentColor" />
      </svg>
    ),
  },
  {
    title: "Carrier adapters",
    description:
      "One integration surface, many carriers. Africa's Talking, Celcom Africa, and direct Safaricom Daraja for M-Pesa STK push.",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6">
        <circle cx="12" cy="12" r="2.4" />
        <circle cx="4.5" cy="6" r="2.4" />
        <circle cx="19.5" cy="6" r="2.4" />
        <circle cx="4.5" cy="18" r="2.4" />
        <circle cx="19.5" cy="18" r="2.4" />
        <path d="M6.4 7.4 10 10.6M17.6 7.4 14 10.6M6.4 16.6 10 13.4M17.6 16.6 14 13.4" />
      </svg>
    ),
  },
  {
    title: "Sessions in Redis",
    description:
      "Multi-step inputs cached with TTLs aligned to carrier timeouts. Sessions survive concurrent callbacks, every time.",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6">
        <ellipse cx="12" cy="5" rx="8" ry="2.6" />
        <path d="M4 5v6c0 1.4 3.6 2.6 8 2.6s8-1.2 8-2.6V5" />
        <path d="M4 11v6c0 1.4 3.6 2.6 8 2.6s8-1.2 8-2.6v-6" />
      </svg>
    ),
  },
  {
    title: "Async webhooks",
    description:
      "Completed flows are relayed to your backend out-of-band, with retries and idempotency keys. The 180-second window is never your problem.",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6">
        <path d="M13 2 4.5 13.5H11l-1 8.5L18.5 10H12l1-8Z" />
      </svg>
    ),
  },
  {
    title: "Live traffic monitor",
    description:
      "Watch sessions, payloads, and M-Pesa callbacks in real time from the dashboard. Debug without guessing.",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" className="h-6 w-6">
        <path d="M3 17l5-6 4 3 6-8" />
        <path d="M14 6h4v4" />
      </svg>
    ),
  },
];

const STEPS = [
  {
    step: "01",
    title: "Define your flow",
    description:
      "Write a menu schema in JSON or YAML: options, inputs, validation, branching. Version it like code.",
  },
  {
    step: "02",
    title: "Deploy a shortcode",
    description:
      "KagoRoute routes your schema through local aggregators. Dial *123# and your flow is live in minutes, not weeks.",
  },
  {
    step: "03",
    title: "Users transact",
    description:
      "Every completed session syncs to your backend and ledger. Smartphone and feature-phone users finally share one system.",
  },
];

const LIVE_SESSIONS = [
  { phone: "254722••••301", flow: "Airtime top-up", amount: "KES 500", status: "complete" },
  { phone: "254733••••812", flow: "Balance check", amount: "—", status: "live" },
  { phone: "254721••••055", flow: "M-Pesa STK push", amount: "KES 1,200", status: "complete" },
  { phone: "254715••••669", flow: "Order fulfilment", amount: "KES 3,450", status: "live" },
];

const USSD_LINES = [
  { sender: "user", text: "*123#" },
  { sender: "ussd", text: "CON Welcome to KagoRoute demo!\n    1. Check balance\n    2. Buy airtime\n    0. Exit" },
  { sender: "user", text: "2" },
  { sender: "ussd", text: "CON Enter airtime amount in KES:" },
  { sender: "user", text: "500" },
  { sender: "ussd", text: "END Airtime of KES 500 queued for processing." },
];

function RouteLogo({ className = "h-8 w-8" }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" className={className} aria-hidden="true">
      <defs>
        <linearGradient id="kago-logo-g" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#34d399" />
          <stop offset="1" stopColor="#0d9488" />
        </linearGradient>
      </defs>
      <rect width="64" height="64" rx="14" fill="currentColor" className="text-[#0d1513]" />
      <circle cx="18" cy="20" r="5" fill="#34d399" />
      <circle cx="46" cy="44" r="5" fill="#0d9488" />
      <path
        d="M23 20 H38 Q46 20 46 26 V39"
        fill="none"
        stroke="url(#kago-logo-g)"
        strokeWidth="4"
        strokeLinecap="round"
      />
      <path
        d="M18 20 L18 32 Q18 40 26 40 H41"
        fill="none"
        stroke="url(#kago-logo-g)"
        strokeWidth="4"
        strokeLinecap="round"
        strokeDasharray="2 8"
      />
    </svg>
  );
}

export default function Home() {
  return (
    <div className="relative overflow-x-clip">
      {/* ── Nav ─────────────────────────────────────────────────────────── */}
      <header className="sticky top-0 z-40 border-b border-white/5 bg-[#060a09]/80 backdrop-blur-md">
        <nav className="mx-auto flex h-16 max-w-6xl items-center justify-between px-6">
          <a href="#" className="flex items-center gap-2.5">
            <RouteLogo />
            <span className="text-lg font-semibold tracking-tight text-white">
              Kago<span className="text-emerald-400">Route</span>
            </span>
          </a>
          <div className="hidden items-center gap-8 text-sm text-zinc-400 md:flex">
            {NAV_LINKS.map((link) => (
              <a
                key={link}
                href="#"
                className="transition-colors hover:text-white"
              >
                {link}
              </a>
            ))}
          </div>
          <Link
            href="/"
            className="rounded-lg bg-emerald-500 px-4 py-2 text-sm font-semibold text-emerald-950 shadow-[0_0_24px_-6px_rgba(52,211,153,0.6)] transition-all hover:bg-emerald-400 hover:shadow-[0_0_28px_-4px_rgba(52,211,153,0.8)]"
          >
            Open dashboard
          </Link>
        </nav>
      </header>

      {/* ── Hero ────────────────────────────────────────────────────────── */}
      <section className="kr-glow relative">
        <div className="mx-auto grid max-w-6xl gap-14 px-6 pb-24 pt-20 lg:grid-cols-2 lg:items-center lg:pt-28">
          <div>
            <div className="inline-flex items-center gap-2 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-xs font-medium text-emerald-300">
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
                <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-400" />
              </span>
              Private beta · Kenya
            </div>

            <h1 className="mt-6 text-4xl font-bold leading-[1.08] tracking-tight text-white sm:text-5xl lg:text-6xl">
              Mirror your app onto{" "}
              <span className="bg-gradient-to-r from-emerald-300 via-emerald-400 to-teal-400 bg-clip-text text-transparent">
                every phone.
              </span>
            </h1>

            <p className="mt-6 max-w-xl text-lg leading-relaxed text-zinc-400">
              KagoRoute turns your existing web and smartphone logic into USSD and
              SMS flows for feature phones — <em className="text-zinc-300 not-italic">kabambe</em>.
              One schema, one ledger, every device.
            </p>

            <div className="mt-8 flex flex-wrap items-center gap-4">
              <Link
                href="/"
                className="rounded-xl bg-emerald-500 px-6 py-3 text-sm font-semibold text-emerald-950 shadow-lg shadow-emerald-500/20 transition-all hover:-translate-y-0.5 hover:bg-emerald-400"
              >
                Open dashboard
              </Link>
              <a
                href="/landing"
                className="rounded-xl border border-white/10 bg-white/5 px-6 py-3 text-sm font-semibold text-white transition-all hover:-translate-y-0.5 hover:border-white/20 hover:bg-white/10"
              >
                Learn more
              </a>
            </div>

            <dl className="mt-12 grid max-w-md grid-cols-3 gap-6">
              {[
                { value: "30 min", label: "to first flow" },
                { value: "180 s", label: "timeouts handled" },
                { value: "1", label: "schema, every phone" },
              ].map((stat) => (
                <div key={stat.label}>
                  <dt className="sr-only">{stat.label}</dt>
                  <dd className="text-2xl font-bold text-white">{stat.value}</dd>
                  <dd className="mt-1 text-xs uppercase tracking-wide text-zinc-500">
                    {stat.label}
                  </dd>
                </div>
              ))}
            </dl>
          </div>

          {/* USSD terminal mock */}
          <div className="relative">
            <div className="absolute -inset-6 rounded-3xl bg-emerald-500/10 blur-3xl" aria-hidden="true" />
            <div className="relative rounded-2xl border border-white/10 bg-[#0b1210] shadow-2xl">
              <div className="flex items-center justify-between border-b border-white/5 px-4 py-3">
                <div className="flex items-center gap-1.5">
                  <span className="h-2.5 w-2.5 rounded-full bg-red-400/80" />
                  <span className="h-2.5 w-2.5 rounded-full bg-amber-400/80" />
                  <span className="h-2.5 w-2.5 rounded-full bg-emerald-400/80" />
                </div>
                <span className="font-mono text-xs text-zinc-500">*123# · USSD session</span>
              </div>
              <div className="space-y-3 p-5 font-mono text-sm leading-relaxed">
                {USSD_LINES.map((line, i) => (
                  <div
                    key={i}
                    className={
                      line.sender === "user"
                        ? "text-zinc-300"
                        : "text-emerald-300"
                    }
                  >
                    <span className="mr-2 text-zinc-600">
                      {line.sender === "user" ? "you" : "ussd"}
                    </span>
                    <span className="whitespace-pre-line">{line.text}</span>
                  </div>
                ))}
                <div className="text-emerald-300">
                  <span className="mr-2 text-zinc-600">ussd</span>
                  <span className="kr-cursor" />
                </div>
              </div>
            </div>

            <div className="mt-4 flex items-center justify-center gap-6 text-xs text-zinc-500">
              <span className="flex items-center gap-1.5">
                <span className="h-1.5 w-1.5 rounded-full bg-zinc-600" /> Africa&apos;s Talking
              </span>
              <span className="flex items-center gap-1.5">
                <span className="h-1.5 w-1.5 rounded-full bg-zinc-600" /> Celcom Africa
              </span>
              <span className="flex items-center gap-1.5">
                <span className="h-1.5 w-1.5 rounded-full bg-zinc-600" /> Safaricom Daraja
              </span>
            </div>
          </div>
        </div>
      </section>

      {/* ── Features ────────────────────────────────────────────────────── */}
      <section className="mx-auto max-w-6xl px-6 py-20">
        <div className="max-w-2xl">
          <p className="text-sm font-semibold uppercase tracking-widest text-emerald-400">
            Why KagoRoute
          </p>
          <h2 className="mt-3 text-3xl font-bold tracking-tight text-white sm:text-4xl">
            The infrastructure layer for the phones everyone still uses
          </h2>
        </div>

        <div className="mt-12 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {FEATURES.map((feature) => (
            <div
              key={feature.title}
              className="group rounded-2xl border border-white/5 bg-white/[0.03] p-6 transition-all hover:-translate-y-1 hover:border-emerald-500/30 hover:bg-white/[0.05]"
            >
              <div className="flex h-11 w-11 items-center justify-center rounded-xl border border-emerald-500/20 bg-emerald-500/10 text-emerald-400 transition-transform group-hover:scale-110">
                {feature.icon}
              </div>
              <h3 className="mt-5 text-lg font-semibold text-white">
                {feature.title}
              </h3>
              <p className="mt-2 text-sm leading-relaxed text-zinc-400">
                {feature.description}
              </p>
            </div>
          ))}
        </div>
      </section>

      {/* ── How it works ────────────────────────────────────────────────── */}
      <section className="border-y border-white/5 bg-white/[0.02]">
        <div className="mx-auto max-w-6xl px-6 py-20">
          <h2 className="text-center text-3xl font-bold tracking-tight text-white sm:text-4xl">
            Three steps to every phone
          </h2>
          <div className="mt-12 grid gap-10 md:grid-cols-3">
            {STEPS.map((step, i) => (
              <div key={step.step} className="relative">
                {i < STEPS.length - 1 && (
                  <div
                    className="absolute left-0 right-0 top-6 hidden h-px bg-gradient-to-r from-emerald-500/40 to-transparent md:block"
                    aria-hidden="true"
                  />
                )}
                <span className="relative inline-flex h-12 w-12 items-center justify-center rounded-full border border-emerald-500/30 bg-[#060a09] font-mono text-sm font-semibold text-emerald-400">
                  {step.step}
                </span>
                <h3 className="mt-5 text-lg font-semibold text-white">
                  {step.title}
                </h3>
                <p className="mt-2 text-sm leading-relaxed text-zinc-400">
                  {step.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── Live traffic panel ──────────────────────────────────────────── */}
      <section className="mx-auto max-w-6xl px-6 py-20">
        <div className="grid gap-10 lg:grid-cols-5 lg:items-center">
          <div className="lg:col-span-2">
            <p className="text-sm font-semibold uppercase tracking-widest text-emerald-400">
              Live traffic
            </p>
            <h2 className="mt-3 text-3xl font-bold tracking-tight text-white">
              Watch sessions as they happen
            </h2>
            <p className="mt-4 text-sm leading-relaxed text-zinc-400">
              Every dial, keystroke, and M-Pesa callback streamed to your
              dashboard in real time. Debug a live flow without guessing what
              the carrier received.
            </p>
          </div>

          <div className="overflow-hidden rounded-2xl border border-white/10 bg-[#0b1210] lg:col-span-3">
            <div className="flex items-center justify-between border-b border-white/5 px-5 py-3">
              <span className="flex items-center gap-2 text-sm font-medium text-white">
                <span className="relative flex h-2 w-2">
                  <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75" />
                  <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-400" />
                </span>
                Active sessions
              </span>
              <span className="font-mono text-xs text-zinc-500">engine · :8080</span>
            </div>
            <table className="w-full text-left text-sm">
              <thead>
                <tr className="border-b border-white/5 text-xs uppercase tracking-wide text-zinc-500">
                  <th className="px-5 py-3 font-medium">Phone</th>
                  <th className="px-5 py-3 font-medium">Flow</th>
                  <th className="px-5 py-3 font-medium">Value</th>
                  <th className="px-5 py-3 font-medium">Status</th>
                </tr>
              </thead>
              <tbody>
                {LIVE_SESSIONS.map((session) => (
                  <tr
                    key={session.phone}
                    className="border-b border-white/5 last:border-0 hover:bg-white/[0.03]"
                  >
                    <td className="px-5 py-3.5 font-mono text-xs text-zinc-300">
                      {session.phone}
                    </td>
                    <td className="px-5 py-3.5 text-zinc-400">{session.flow}</td>
                    <td className="px-5 py-3.5 text-zinc-300">{session.amount}</td>
                    <td className="px-5 py-3.5">
                      {session.status === "live" ? (
                        <span className="inline-flex items-center gap-1.5 rounded-full bg-emerald-500/10 px-2.5 py-0.5 text-xs font-medium text-emerald-300">
                          <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-emerald-400" />
                          live
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1.5 rounded-full bg-white/5 px-2.5 py-0.5 text-xs font-medium text-zinc-400">
                          complete
                        </span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* ── CTA ─────────────────────────────────────────────────────────── */}
      <section className="mx-auto max-w-6xl px-6 pb-24">
        <div className="relative overflow-hidden rounded-3xl border border-emerald-500/20 bg-gradient-to-br from-emerald-500/10 via-[#0b1210] to-[#060a09] px-8 py-16 text-center">
          <div
            className="absolute inset-0 opacity-40"
            style={{
              backgroundImage:
                "radial-gradient(circle at 20% 30%, rgba(52,211,153,0.15), transparent 40%), radial-gradient(circle at 80% 70%, rgba(13,148,136,0.15), transparent 40%)",
            }}
            aria-hidden="true"
          />
          <div className="relative">
            <h2 className="text-3xl font-bold tracking-tight text-white sm:text-4xl">
              Your app already exists. Your customers use kabambe.
            </h2>
            <p className="mx-auto mt-4 max-w-xl text-zinc-400">
              Bridge the gap in minutes. KagoRoute keeps one ledger, one
              inventory, one account — across every screen in Kenya.
            </p>
            <a
              href="#"
              className="mt-8 inline-block rounded-xl bg-emerald-500 px-8 py-3.5 text-sm font-semibold text-emerald-950 shadow-lg shadow-emerald-500/25 transition-all hover:-translate-y-0.5 hover:bg-emerald-400"
            >
              Request beta access
            </a>
          </div>
        </div>
      </section>

      {/* ── Footer ──────────────────────────────────────────────────────── */}
      <footer className="border-t border-white/5">
        <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-6 py-8 text-sm text-zinc-500 sm:flex-row">
          <div className="flex items-center gap-2.5">
            <RouteLogo className="h-6 w-6" />
            <span className="text-zinc-400">
              Kago<span className="text-emerald-400">Route</span>
            </span>
          </div>
          <p>© 2026 KagoRoute. Built for the phones Kenya actually uses.</p>
        </div>
      </footer>
    </div>
  );
}

"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { ENGINE_URL } from "@/lib/api";

export const NAV = [
  { label: "Overview", href: "/", soon: false },
  { label: "Flows", href: "/flows", soon: false },
  { label: "Sessions", href: "#", soon: true },
  { label: "Webhooks", href: "#", soon: true },
  { label: "Settings", href: "#", soon: true },
];

export function Sidebar() {
  const pathname = usePathname();
  return (
    <aside className="hidden w-60 shrink-0 flex-col border-r border-white/5 bg-white/[0.015] lg:flex">
      <Link href="/" className="flex items-center gap-2.5 px-6 py-5">
        <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-emerald-400 to-teal-500 text-sm font-bold text-emerald-950">
          K
        </span>
        <span className="text-base font-semibold tracking-tight text-white">
          Kago<span className="text-emerald-400">Route</span>
        </span>
      </Link>

      <nav className="mt-2 flex-1 space-y-1 px-3">
        {NAV.map((item) => {
          const active = pathname === item.href;
          return (
            <Link
              key={item.label}
              href={item.href}
              aria-current={active ? "page" : undefined}
              className={`flex items-center justify-between rounded-lg px-3 py-2 text-sm transition-colors ${
                active
                  ? "bg-emerald-500/10 font-medium text-emerald-300"
                  : "text-zinc-400 hover:bg-white/5 hover:text-white"
              }`}
            >
              <span>{item.label}</span>
              {item.soon ? (
                <span className="rounded-full border border-white/10 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-zinc-500">
                  soon
                </span>
              ) : null}
            </Link>
          );
        })}
      </nav>

      <div className="border-t border-white/5 px-6 py-4">
        <p className="text-[11px] uppercase tracking-wider text-zinc-600">
          Engine
        </p>
        <p className="mt-1 truncate font-mono text-xs text-zinc-400">
          {ENGINE_URL}
        </p>
        <Link
          href="/landing"
          className="mt-3 inline-block text-xs text-zinc-500 transition-colors hover:text-emerald-400"
        >
          ← Marketing site
        </Link>
      </div>
    </aside>
  );
}

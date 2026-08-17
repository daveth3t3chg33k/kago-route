"use client";

import { summarizeIssues, type ValidationIssue } from "@/lib/schema/validate";

export function ValidationPanel({ issues }: { issues: ValidationIssue[] }) {
  const { total, schema, semantic } = summarizeIssues(issues);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-white">Validation</span>
        <span
          className={`rounded-full px-2 py-0.5 font-mono text-xs ${
            total === 0
              ? "border border-emerald-500/30 bg-emerald-500/10 text-emerald-300"
              : "border border-red-500/30 bg-red-500/10 text-red-300"
          }`}
        >
          {total === 0 ? "valid" : `${total} issue${total > 1 ? "s" : ""}`}
        </span>
      </div>

      {total === 0 ? (
        <p className="rounded-lg border border-emerald-500/20 bg-emerald-500/5 px-3 py-2 text-xs text-emerald-300">
          ✓ Conforms to the KagoRoute menu schema and the flow is traversable.
        </p>
      ) : (
        <ul className="space-y-1.5">
          {issues.map((issue, i) => (
            <li
              key={i}
              className="rounded-lg border border-red-500/20 bg-red-500/5 px-3 py-2 text-xs"
            >
              <span className="font-mono text-[10px] text-zinc-500">
                {issue.kind === "schema" ? "schema" : "flow"}
                {issue.path ? ` · ${issue.path}` : ""}
              </span>
              <p className="mt-0.5 text-red-200">{issue.message}</p>
            </li>
          ))}
        </ul>
      )}

      <p className="text-[11px] leading-relaxed text-zinc-600">
        Checked against docs/menu-schema.schema.json ({schema} schema · {semantic} flow
        issues). The engine runs the same rules fail-closed at boot.
      </p>
    </div>
  );
}

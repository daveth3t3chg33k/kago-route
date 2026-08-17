"use client";

import { useState } from "react";
import type { FlowDocument } from "@/lib/schema/types";
import { toJson, toYaml } from "@/lib/schema/serialize";

export function OutputPanel({ doc }: { doc: FlowDocument }) {
  const [format, setFormat] = useState<"json" | "yaml">("yaml");
  const [copied, setCopied] = useState(false);

  const text = format === "json" ? toJson(doc) : toYaml(doc);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // Clipboard unavailable (non-secure context) — fall back to select-all.
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }
  };

  const download = () => {
    const blob = new Blob([text], { type: format === "json" ? "application/json" : "text/yaml" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${doc.flow.id || "flow"}.${format}`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex rounded-lg border border-white/10 p-0.5">
          {(["yaml", "json"] as const).map((f) => (
            <button
              key={f}
              type="button"
              onClick={() => setFormat(f)}
              className={`rounded-md px-3 py-1 font-mono text-xs uppercase transition-colors ${
                format === f
                  ? "bg-emerald-500/20 text-emerald-300"
                  : "text-zinc-500 hover:text-zinc-300"
              }`}
            >
              {f}
            </button>
          ))}
        </div>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={download}
            className="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-xs text-zinc-300 transition-colors hover:border-emerald-500/40 hover:text-white"
          >
            Download
          </button>
          <button
            type="button"
            onClick={copy}
            className={`rounded-lg px-2.5 py-1 text-xs font-medium transition-colors ${
              copied
                ? "border border-emerald-500/40 bg-emerald-500/20 text-emerald-300"
                : "border border-emerald-500/40 bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/20"
            }`}
          >
            {copied ? "Copied!" : "Copy"}
          </button>
        </div>
      </div>

      <pre className="max-h-[60vh] overflow-auto rounded-xl border border-white/10 bg-[#070c0a] p-4 font-mono text-xs leading-relaxed text-emerald-100/90">
        {text}
      </pre>

      <p className="text-[11px] leading-relaxed text-zinc-600">
        This document round-trips through the engine: set <code className="text-zinc-400">FLOW_SCHEMA_PATH</code> to
        it and the same schema is deployed.
      </p>
    </div>
  );
}

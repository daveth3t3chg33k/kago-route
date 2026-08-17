"use client";

import type { Node } from "@/lib/schema/types";
import { branchesOf } from "@/lib/schema/types";

const TYPE_STYLES: Record<Node["type"], { label: string; chip: string; border: string }> = {
  menu: {
    label: "menu",
    chip: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
    border: "hover:border-emerald-500/40",
  },
  input: {
    label: "input",
    chip: "bg-sky-500/15 text-sky-300 border-sky-500/30",
    border: "hover:border-sky-500/40",
  },
  action: {
    label: "action",
    chip: "bg-violet-500/15 text-violet-300 border-violet-500/30",
    border: "hover:border-violet-500/40",
  },
  end: {
    label: "end",
    chip: "bg-amber-500/15 text-amber-300 border-amber-500/30",
    border: "hover:border-amber-500/40",
  },
};

function truncated(text: string, max = 90): string {
  const oneLine = text.replace(/\n/g, " · ");
  return oneLine.length > max ? oneLine.slice(0, max - 1) + "…" : oneLine;
}

export function NodeCard({
  id,
  node,
  selected,
  onSelect,
}: {
  id: string;
  node: Node;
  selected: boolean;
  onSelect: (id: string) => void;
}) {
  const style = TYPE_STYLES[node.type];

  return (
    <button
      type="button"
      onClick={(e) => {
        e.stopPropagation();
        onSelect(id);
      }}
      className={`w-full rounded-xl border bg-[#0d1513] p-3 text-left shadow-lg transition-all ${
        selected
          ? "border-emerald-400 ring-2 ring-emerald-400/30"
          : `border-white/10 ${style.border}`
      }`}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="truncate font-mono text-xs font-semibold text-white">{id}</span>
        <span
          className={`shrink-0 rounded-full border px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide ${style.chip}`}
        >
          {style.label}
        </span>
      </div>

      <div className="mt-2 space-y-1 text-xs leading-relaxed text-zinc-400">
        {node.type === "menu" && (
          <>
            <p className="text-zinc-300">{truncated(node.text)}</p>
            <div className="flex flex-wrap gap-1 pt-0.5">
              {Object.entries(node.options).map(([digit, opt]) => (
                <span
                  key={digit}
                  className="rounded border border-white/10 bg-white/5 px-1 font-mono text-[10px] text-emerald-300"
                >
                  {digit}→{branchesOf(opt).map((b) => b.goto).join("/")}
                </span>
              ))}
              {Object.keys(node.options).length === 0 && (
                <span className="text-[10px] italic text-zinc-600">no options</span>
              )}
            </div>
          </>
        )}
        {node.type === "input" && (
          <>
            <p className="text-zinc-300">{truncated(node.prompt)}</p>
            <p className="font-mono text-[10px] text-sky-300">
              {node.variable}
              {node.validate ? ` · ${node.validate.type}` : ""}
            </p>
          </>
        )}
        {node.type === "action" && (
          <>
            <p className="font-mono text-[10px] text-violet-300">
              {Object.keys(node.compute ?? {}).length > 0 &&
                `compute: ${Object.entries(node.compute!).map(([k, v]) => `${k}=${v}`).join(", ")}`}
            </p>
            <p className="font-mono text-[10px] text-violet-300">
              {Object.keys(node.set ?? {}).length > 0 &&
                `set: ${Object.keys(node.set!).join(", ")}`}
            </p>
          </>
        )}
        {node.type === "end" && <p className="text-zinc-300">{truncated(node.text)}</p>}
      </div>
    </button>
  );
}

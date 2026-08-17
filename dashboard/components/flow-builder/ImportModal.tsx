"use client";

import { useState } from "react";
import { parse as yamlParse } from "yaml";
import { blankDocument } from "@/lib/schema/serialize";
import type { FlowDocument } from "@/lib/schema/types";
import { DSL_IDENTIFIER } from "@/lib/schema/types";
import { validateDocument } from "@/lib/schema/validate";

export function ImportModal({
  onClose,
  onImport,
}: {
  onClose: () => void;
  onImport: (doc: FlowDocument) => void;
}) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [warning, setWarning] = useState<string | null>(null);
  const [parsed, setParsed] = useState<FlowDocument | null>(null);

  const parse = () => {
    setError(null);
    setWarning(null);
    setParsed(null);
    if (!text.trim()) {
      setError("Paste a schema first.");
      return;
    }
    let doc: unknown;
    try {
      doc = JSON.parse(text);
    } catch {
      try {
        doc = yamlParse(text);
      } catch {
        setError("Could not parse — not valid JSON or YAML.");
        return;
      }
    }
    const candidate = doc as Partial<FlowDocument>;
    if (candidate?.schema !== DSL_IDENTIFIER) {
      setError(`Expected schema "${DSL_IDENTIFIER}" — got "${candidate?.schema ?? "nothing"}".`);
      return;
    }
    if (!candidate.flow) {
      setError("Missing 'flow' object.");
      return;
    }
    const issues = validateDocument(doc);
    if (issues.some((i) => i.kind === "schema")) {
      setError("Schema does not conform to docs/menu-schema.schema.json (see list below).");
    } else if (issues.length > 0) {
      setWarning("Conforms to the schema, but has flow-level issues you may want to fix.");
    }
    setParsed(doc as FlowDocument);
  };

  const apply = () => {
    if (parsed) onImport(parsed);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm" onClick={onClose}>
      <div
        className="w-full max-w-2xl rounded-2xl border border-white/10 bg-[#0b1210] p-5 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold text-white">Import schema</h2>
          <button type="button" onClick={onClose} className="rounded p-1 text-zinc-500 hover:text-white">
            ✕
          </button>
        </div>
        <p className="mt-1 text-xs text-zinc-500">
          Paste a YAML or JSON menu schema (e.g. an example from docs/examples/). It will replace
          the current flow in the editor.
        </p>
        <textarea
          value={text}
          onChange={(e) => setText(e.target.value)}
          rows={14}
          spellCheck={false}
          placeholder={'schema: "kagoroute/menu/1.0"\nflow:\n  id: my-flow\n  …'}
          className="mt-3 w-full resize-y rounded-xl border border-white/10 bg-[#070c0a] p-3 font-mono text-xs text-emerald-100/90 outline-none placeholder:text-zinc-600 focus:border-emerald-500/50"
        />

        {error && (
          <p className="mt-2 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-200">
            {error}
          </p>
        )}
        {warning && (
          <p className="mt-2 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-xs text-amber-200">
            {warning}
          </p>
        )}
        {parsed && !error && (
          <p className="mt-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-200">
            ✓ Parsed: flow <span className="font-mono">{parsed.flow.id}</span> v{parsed.flow.version} ·{" "}
            {Object.keys(parsed.flow.nodes).length} nodes
          </p>
        )}

        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            onClick={() => {
              setText("");
              setError(null);
              setParsed(null);
            }}
            className="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-zinc-300 hover:text-white"
          >
            Clear
          </button>
          <button
            type="button"
            onClick={parse}
            className="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-zinc-300 hover:text-white"
          >
            Parse
          </button>
          <button
            type="button"
            onClick={apply}
            disabled={!parsed}
            className="rounded-lg bg-emerald-500 px-3 py-1.5 text-xs font-semibold text-emerald-950 transition-colors hover:bg-emerald-400 disabled:cursor-not-allowed disabled:opacity-40"
          >
            Load into editor
          </button>
        </div>
      </div>
    </div>
  );
}

export { blankDocument };

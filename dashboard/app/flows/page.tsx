"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { engineApi } from "@/lib/api";
import { Sidebar } from "@/components/Sidebar";
import { FlowCanvas } from "@/components/flow-builder/FlowCanvas";
import { FlowInspector, NodeInspector } from "@/components/flow-builder/Inspector";
import { ValidationPanel } from "@/components/flow-builder/ValidationPanel";
import { OutputPanel } from "@/components/flow-builder/OutputPanel";
import { ImportModal } from "@/components/flow-builder/ImportModal";
import type { FlowDocument, Node, NodeType } from "@/lib/schema/types";
import { autoLayout, type Point } from "@/lib/schema/graph";
import { blankDocument } from "@/lib/schema/serialize";
import { validateDocument } from "@/lib/schema/validate";

type Tab = "edit" | "validate" | "output";

function newNode(type: NodeType): Node {
  switch (type) {
    case "menu":
      return { type: "menu", text: "New menu", options: { "1": { goto: "" } } };
    case "input":
      return { type: "input", prompt: "Enter a value:", variable: "value", next: "" };
    case "action":
      return { type: "action", set: {}, compute: {}, next: "" };
    case "end":
      return { type: "end", text: "Thank you. Goodbye." };
  }
}

export default function FlowsPage() {
  const [doc, setDoc] = useState<FlowDocument | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [positions, setPositions] = useState<Record<string, Point>>({});
  const [tab, setTab] = useState<Tab>("edit");
  const [importOpen, setImportOpen] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Load the loaded engine schema on mount; fall back to a blank document.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const schema = await engineApi.flowSchema();
        if (!cancelled) {
          setDoc(schema);
          setPositions(autoLayout(schema.flow.nodes, schema.flow.start));
        }
      } catch (e) {
        if (!cancelled) {
          setLoadError((e as Error).message);
          const blank = blankDocument();
          setDoc(blank);
          setPositions(autoLayout(blank.flow.nodes, blank.flow.start));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const flow = doc?.flow;
  const nodeIds = useMemo(() => (flow ? Object.keys(flow.nodes) : []), [flow]);
  const issues = useMemo(
    () => (doc ? validateDocument(doc) : []),
    [doc],
  );

  const updateFlow = useCallback((next: FlowDocument) => {
    setDoc(next);
    // New nodes (added elsewhere) get a position near the start.
    setPositions((prev) => {
      const merged = { ...prev };
      for (const id of Object.keys(next.flow.nodes)) {
        if (!merged[id]) {
          const base = merged[next.flow.start] ?? { x: 0, y: 0 };
          merged[id] = { x: base.x + 40, y: base.y + 40 };
        }
      }
      return merged;
    });
  }, []);

  const patchNode = useCallback(
    (id: string, node: Node) => {
      if (!doc) return;
      updateFlow({
        ...doc,
        flow: { ...doc.flow, nodes: { ...doc.flow.nodes, [id]: node } },
      });
    },
    [doc, updateFlow],
  );

  const deleteNode = useCallback(
    (id: string) => {
      if (!doc) return;
      const nodes = { ...doc.flow.nodes };
      delete nodes[id];
      // Re-point any dangling references at the start node.
      for (const n of Object.values(nodes)) {
        if (n.type === "menu") {
          for (const [digit, opt] of Object.entries(n.options)) {
            const branches = Array.isArray(opt) ? opt : [opt];
            const fixed = branches.map((b) =>
              b.goto === id ? { ...b, goto: doc.flow.start } : b,
            );
            n.options[digit] = fixed.length === 1 ? fixed[0] : fixed;
          }
          if (n.onInvalid?.goto === id) n.onInvalid = { ...n.onInvalid, goto: doc.flow.start };
          if (n.onTimeout?.goto === id) n.onTimeout = { ...n.onTimeout, goto: doc.flow.start };
        } else if (n.type === "input") {
          if (n.next === id) n.next = doc.flow.start;
          if (n.onInvalid?.goto === id) n.onInvalid = { ...n.onInvalid, goto: doc.flow.start };
          if (n.onTimeout?.goto === id) n.onTimeout = { ...n.onTimeout, goto: doc.flow.start };
        } else if (n.type === "action") {
          if (n.next === id) n.next = doc.flow.start;
        } else if (n.type === "end" && n.onTimeout?.goto === id) {
          n.onTimeout = { ...n.onTimeout, goto: doc.flow.start };
        }
      }
      const next = { ...doc, flow: { ...doc.flow, nodes } };
      setDoc(next);
      setSelected(null);
    },
    [doc],
  );

  const addNode = useCallback(
    (type: NodeType) => {
      if (!doc) return;
      const id = `${type}-${Date.now().toString(36).slice(-4)}`;
      updateFlow({
        ...doc,
        flow: {
          ...doc.flow,
          nodes: { ...doc.flow.nodes, [id]: newNode(type) },
        },
      });
      setSelected(id);
    },
    [doc, updateFlow],
  );

  const redoLayout = useCallback(() => {
    if (!doc) return;
    setPositions(autoLayout(doc.flow.nodes, doc.flow.start));
  }, [doc]);

  const onImport = useCallback((incoming: FlowDocument) => {
    setDoc(incoming);
    setPositions(autoLayout(incoming.flow.nodes, incoming.flow.start));
    setImportOpen(false);
    setSelected(null);
  }, []);

  if (!doc) {
    return (
      <div className="flex min-h-screen">
        <Sidebar />
        <main className="flex flex-1 items-center justify-center">
          <p className="text-sm text-zinc-500">
            {loadError ? "Starting with a blank flow…" : "Loading schema…"}
          </p>
        </main>
      </div>
    );
  }

  const issueCount = issues.length;

  return (
    <div className="flex min-h-screen">
      <Sidebar />

      <main className="flex min-w-0 flex-1 flex-col">
        {/* Toolbar */}
        <header className="sticky top-0 z-40 border-b border-white/5 bg-[#060a09]/85 backdrop-blur-md">
          <div className="flex flex-wrap items-center justify-between gap-3 px-6 py-3">
            <div className="flex items-center gap-3">
              <h1 className="text-base font-semibold tracking-tight text-white">Flow builder</h1>
              <span className="rounded-full border border-white/10 bg-white/5 px-2 py-0.5 font-mono text-xs text-zinc-300">
                {flow?.id ?? "…"}
              </span>
              <span className="hidden rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 font-mono text-xs text-emerald-300 sm:inline">
                v{flow?.version ?? "?"}
              </span>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              <button
                type="button"
                onClick={redoLayout}
                className="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-zinc-300 transition-colors hover:border-emerald-500/40 hover:text-white"
              >
                Auto-layout
              </button>
              <button
                type="button"
                onClick={() => setImportOpen(true)}
                className="rounded-lg border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-zinc-300 transition-colors hover:border-emerald-500/40 hover:text-white"
              >
                Import
              </button>
              <div className="mx-1 h-5 w-px bg-white/10" />
              {(["menu", "input", "action", "end"] as const).map((t) => (
                <button
                  key={t}
                  type="button"
                  onClick={() => addNode(t)}
                  className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-1.5 text-xs font-medium text-emerald-300 transition-colors hover:bg-emerald-500/20"
                >
                  + {t}
                </button>
              ))}
            </div>
          </div>

          {/* Tabs */}
          <div className="flex gap-1 px-6 pb-2">
            {(
              [
                ["edit", "Editor"],
                ["validate", `Validation${issueCount ? ` (${issueCount})` : ""}`],
                ["output", "YAML / JSON"],
              ] as const
            ).map(([key, label]) => (
              <button
                key={key}
                type="button"
                onClick={() => setTab(key)}
                className={`rounded-t-lg border-b-2 px-3 py-1.5 text-xs font-medium transition-colors ${
                  tab === key
                    ? "border-emerald-400 text-white"
                    : "border-transparent text-zinc-500 hover:text-zinc-300"
                }`}
              >
                {label}
              </button>
            ))}
          </div>
        </header>

        <div className="flex min-h-0 flex-1 flex-col gap-4 p-4 lg:flex-row">
          {/* Canvas */}
          <div className="min-h-[420px] flex-1 lg:min-h-0">
            <FlowCanvas
              nodes={flow?.nodes ?? {}}
              positions={positions}
              selectedId={selected}
              onSelect={(id) => setSelected(id)}
              onMove={(id, pos) =>
                setPositions((prev) => ({ ...prev, [id]: pos }))
              }
            />
          </div>

          {/* Side panel */}
          <aside className="w-full shrink-0 overflow-y-auto rounded-xl border border-white/10 bg-[#0b1210] p-4 lg:w-[380px]">
            {tab === "edit" && (
              <>
                <p className="mb-3 text-[11px] font-medium uppercase tracking-wider text-zinc-500">
                  {selected ? `Node · ${selected}` : "Flow settings"}
                </p>
                {selected && flow?.nodes[selected] ? (
                  <NodeInspector
                    node={flow.nodes[selected]}
                    nodeIds={nodeIds}
                    onChange={(node) => patchNode(selected, node)}
                    onDelete={() => deleteNode(selected)}
                  />
                ) : (
                  <FlowInspector
                    flow={flow!}
                    nodeIds={nodeIds}
                    onChange={(flow) => updateFlow({ ...doc, flow })}
                  />
                )}
              </>
            )}
            {tab === "validate" && <ValidationPanel issues={issues} />}
            {tab === "output" && <OutputPanel doc={doc} />}
          </aside>
        </div>
      </main>

      {importOpen && <ImportModal onClose={() => setImportOpen(false)} onImport={onImport} />}
    </div>
  );
}

"use client";

import type { Condition, Flow, Node, ValidationKind } from "@/lib/schema/types";

const VALIDATION_KINDS: ValidationKind[] = ["int", "float", "text", "phone", "amount", "option"];

function Label({ children }: { children: React.ReactNode }) {
  return (
    <label className="block text-[11px] font-medium uppercase tracking-wider text-zinc-500">
      {children}
    </label>
  );
}

function TextField({
  label,
  value,
  onChange,
  placeholder,
  mono,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  mono?: boolean;
}) {
  return (
    <div>
      <Label>{label}</Label>
      <input
        type="text"
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
        className={`mt-1 w-full rounded-lg border border-white/10 bg-white/5 px-2.5 py-1.5 text-sm text-white outline-none transition-colors placeholder:text-zinc-600 focus:border-emerald-500/50 ${
          mono ? "font-mono text-xs" : ""
        }`}
      />
    </div>
  );
}

function TextArea({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div>
      <Label>{label}</Label>
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        rows={3}
        className="mt-1 w-full resize-y rounded-lg border border-white/10 bg-white/5 px-2.5 py-1.5 text-sm text-white outline-none transition-colors placeholder:text-zinc-600 focus:border-emerald-500/50"
      />
    </div>
  );
}

function NumberField({
  label,
  value,
  onChange,
}: {
  label: string;
  value?: number;
  onChange: (v: number | undefined) => void;
}) {
  return (
    <div className="w-full">
      <Label>{label}</Label>
      <input
        type="number"
        value={value ?? ""}
        onChange={(e) => {
          const raw = e.target.value;
          onChange(raw === "" ? undefined : Number(raw));
        }}
        className="mt-1 w-full rounded-lg border border-white/10 bg-white/5 px-2.5 py-1.5 text-sm text-white outline-none transition-colors focus:border-emerald-500/50"
      />
    </div>
  );
}

function SelectField({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  options: string[];
}) {
  return (
    <div>
      <Label>{label}</Label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="mt-1 w-full rounded-lg border border-white/10 bg-[#0d1513] px-2.5 py-1.5 text-sm text-white outline-none transition-colors focus:border-emerald-500/50"
      >
        {options.map((o) => (
          <option key={o} value={o}>
            {o}
          </option>
        ))}
      </select>
    </div>
  );
}

function NodeSelect({
  label,
  value,
  onChange,
  nodeIds,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  nodeIds: string[];
}) {
  return (
    <div>
      <Label>{label}</Label>
      <select
        value={nodeIds.includes(value) ? value : ""}
        onChange={(e) => onChange(e.target.value)}
        className="mt-1 w-full rounded-lg border border-white/10 bg-[#0d1513] px-2.5 py-1.5 font-mono text-xs text-white outline-none transition-colors focus:border-emerald-500/50"
      >
        <option value="" disabled>
          select node…
        </option>
        {nodeIds.map((id) => (
          <option key={id} value={id}>
            {id}
          </option>
        ))}
      </select>
    </div>
  );
}

function RecoveryEditor({
  label,
  recovery,
  nodeIds,
  onChange,
}: {
  label: string;
  recovery: { text?: string; goto: string };
  nodeIds: string[];
  onChange: (r: { text?: string; goto: string }) => void;
}) {
  return (
    <div className="rounded-lg border border-white/10 bg-white/[0.02] p-2.5">
      <p className="mb-2 text-[11px] font-medium uppercase tracking-wider text-zinc-500">
        {label}
      </p>
      <TextField label="Text" value={recovery.text ?? ""} onChange={(v) => onChange({ ...recovery, text: v || undefined })} placeholder="optional message" />
      <div className="mt-2">
        <NodeSelect label="Goto" value={recovery.goto} onChange={(v) => onChange({ ...recovery, goto: v })} nodeIds={nodeIds} />
      </div>
    </div>
  );
}

function MapEditor({
  label,
  map,
  onChange,
  valuePlaceholder,
  monoValue,
}: {
  label: string;
  map: Record<string, string>;
  onChange: (m: Record<string, string>) => void;
  valuePlaceholder?: string;
  monoValue?: boolean;
}) {
  const entries = Object.entries(map);
  return (
    <div>
      <Label>{label}</Label>
      <div className="mt-1 space-y-1.5">
        {entries.map(([k, v]) => (
          <div key={k} className="flex items-center gap-1.5">
            <input
              type="text"
              value={k}
              onChange={(e) => {
                const next = { ...map };
                delete next[k];
                next[e.target.value] = v;
                onChange(next);
              }}
              placeholder="key"
              className="w-1/3 rounded-lg border border-white/10 bg-white/5 px-2 py-1 font-mono text-xs text-white outline-none focus:border-emerald-500/50"
            />
            <span className="text-zinc-600">=</span>
            <input
              type="text"
              value={v}
              onChange={(e) => onChange({ ...map, [k]: e.target.value })}
              placeholder={valuePlaceholder ?? "value"}
              className={`w-full rounded-lg border border-white/10 bg-white/5 px-2 py-1 text-xs text-white outline-none focus:border-emerald-500/50 ${
                monoValue ? "font-mono" : ""
              }`}
            />
            <button
              type="button"
              onClick={() => {
                const next = { ...map };
                delete next[k];
                onChange(next);
              }}
              className="shrink-0 rounded px-1.5 text-xs text-zinc-500 hover:text-red-400"
              title="Remove"
            >
              ×
            </button>
          </div>
        ))}
        <button
          type="button"
          onClick={() => onChange({ ...map, ["new" + (entries.length + 1)]: "" })}
          className="rounded-lg border border-dashed border-white/15 px-2 py-1 text-xs text-zinc-500 transition-colors hover:border-emerald-500/40 hover:text-emerald-300"
        >
          + add
        </button>
      </div>
    </div>
  );
}

export function FlowInspector({
  flow,
  nodeIds,
  onChange,
}: {
  flow: Flow;
  nodeIds: string[];
  onChange: (flow: Flow) => void;
}) {
  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-3">
        <TextField label="ID" value={flow.id} onChange={(v) => onChange({ ...flow, id: v })} mono />
        <TextField label="Name" value={flow.name} onChange={(v) => onChange({ ...flow, name: v })} />
      </div>
      <TextField label="Description" value={flow.description ?? ""} onChange={(v) => onChange({ ...flow, description: v || undefined })} />
      <div className="grid grid-cols-2 gap-3">
        <NumberField label="Version" value={flow.version} onChange={(v) => onChange({ ...flow, version: v ?? 1 })} />
        <NodeSelect label="Start node" value={flow.start} onChange={(v) => onChange({ ...flow, start: v })} nodeIds={nodeIds} />
      </div>
      <div className="grid grid-cols-2 gap-3">
        <NumberField label="Session (s)" value={flow.timeouts?.session} onChange={(v) => onChange({ ...flow, timeouts: { ...flow.timeouts, session: v } })} />
        <NumberField label="Step (s)" value={flow.timeouts?.step} onChange={(v) => onChange({ ...flow, timeouts: { ...flow.timeouts, step: v } })} />
      </div>
    </div>
  );
}

export function NodeInspector({
  node,
  nodeIds,
  onChange,
  onDelete,
}: {
  node: Node;
  nodeIds: string[];
  onChange: (node: Node) => void;
  onDelete: () => void;
}) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <span className="font-mono text-xs font-semibold uppercase tracking-wide text-zinc-500">
          {node.type} node
        </span>
        <button
          type="button"
          onClick={onDelete}
          className="rounded-lg border border-red-500/30 bg-red-500/10 px-2 py-1 text-xs font-medium text-red-300 transition-colors hover:bg-red-500/20"
        >
          Delete
        </button>
      </div>

      {node.type === "menu" && (
        <MenuEditor
          node={node}
          nodeIds={nodeIds}
          onChange={(n) => onChange(n)}
        />
      )}
      {node.type === "input" && (
        <InputEditor
          node={node}
          nodeIds={nodeIds}
          onChange={(n) => onChange(n)}
        />
      )}
      {node.type === "action" && (
        <ActionEditor node={node} nodeIds={nodeIds} onChange={(n) => onChange(n)} />
      )}
      {node.type === "end" && <EndEditor node={node} onChange={(n) => onChange(n)} />}
    </div>
  );
}

function MenuEditor({
  node,
  nodeIds,
  onChange,
}: {
  node: Extract<Node, { type: "menu" }>;
  nodeIds: string[];
  onChange: (n: Node) => void;
}) {
  const options = Object.entries(node.options);
  return (
    <div className="space-y-3">
      <TextArea label="Text" value={node.text} onChange={(text) => onChange({ ...node, text })} />
      <div>
        <Label>Options</Label>
        <div className="mt-1 space-y-2">
          {options.map(([digit, opt]) => {
            const branches = Array.isArray(opt) ? opt : [opt];
            return (
              <div key={digit} className="rounded-lg border border-white/10 bg-white/[0.02] p-2">
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={digit}
                    onChange={(e) => {
                      const next = { ...node.options };
                      delete next[digit];
                      next[e.target.value] = opt;
                      onChange({ ...node, options: next });
                    }}
                    className="w-10 rounded-lg border border-white/10 bg-white/5 px-1.5 py-1 text-center font-mono text-xs text-white outline-none focus:border-emerald-500/50"
                  />
                  <span className="text-xs text-zinc-600">→</span>
                  <select
                    value={branches[0]?.goto ?? ""}
                    onChange={(e) => {
                      const target = e.target.value;
                      const newBranches = branches.map((b, i) =>
                        i === 0 ? { ...b, goto: target } : b,
                      );
                      const next = { ...node.options, [digit]: newBranches.length === 1 ? newBranches[0] : newBranches };
                      onChange({ ...node, options: next });
                    }}
                    className="w-full rounded-lg border border-white/10 bg-[#0d1513] px-2 py-1 font-mono text-xs text-white outline-none focus:border-emerald-500/50"
                  >
                    <option value="" disabled>
                      select node…
                    </option>
                    {nodeIds.map((nid) => (
                      <option key={nid} value={nid}>
                        {nid}
                      </option>
                    ))}
                  </select>
                  <button
                    type="button"
                    onClick={() => {
                      const next = { ...node.options };
                      delete next[digit];
                      onChange({ ...node, options: next });
                    }}
                    className="shrink-0 rounded px-1.5 text-xs text-zinc-500 hover:text-red-400"
                    title="Remove option"
                  >
                    ×
                  </button>
                </div>
                {branches.map((b, i) =>
                  b.when && conditionsList(b.when).length > 0 ? (
                    <div key={i} className="mt-1.5 flex flex-wrap gap-1">
                      {conditionsList(b.when).map((c, j) => (
                        <span
                          key={j}
                          className="rounded border border-violet-500/30 bg-violet-500/10 px-1.5 py-0.5 font-mono text-[10px] text-violet-300"
                        >
                          {c.var} {c.op} {c.value === undefined ? "" : String(c.value)}
                        </span>
                      ))}
                      <span className="px-1 py-0.5 text-[10px] text-zinc-600">+ {branches.length > 1 ? `${branches.length} branches` : "when-chain"}</span>
                    </div>
                  ) : null,
                )}
              </div>
            );
          })}
          <button
            type="button"
            onClick={() => {
              const digit = String(Object.keys(node.options).length + 1);
              onChange({
                ...node,
                options: { ...node.options, [digit]: { goto: "" } },
              });
            }}
            className="w-full rounded-lg border border-dashed border-white/15 px-2 py-1.5 text-xs text-zinc-500 transition-colors hover:border-emerald-500/40 hover:text-emerald-300"
          >
            + add option
          </button>
        </div>
      </div>
      {node.onInvalid && (
        <RecoveryEditor
          label="onInvalid"
          recovery={node.onInvalid}
          nodeIds={nodeIds}
          onChange={(onInvalid) => onChange({ ...node, onInvalid })}
        />
      )}
      {node.onTimeout && (
        <RecoveryEditor
          label="onTimeout"
          recovery={node.onTimeout}
          nodeIds={nodeIds}
          onChange={(onTimeout) => onChange({ ...node, onTimeout })}
        />
      )}
    </div>
  );
}

function conditionsList(when: Condition | Condition[]): Condition[] {
  return Array.isArray(when) ? when : [when];
}

function InputEditor({
  node,
  nodeIds,
  onChange,
}: {
  node: Extract<Node, { type: "input" }>;
  nodeIds: string[];
  onChange: (n: Node) => void;
}) {
  const validate = node.validate;
  return (
    <div className="space-y-3">
      <TextArea label="Prompt" value={node.prompt} onChange={(prompt) => onChange({ ...node, prompt })} />
      <TextField label="Variable" value={node.variable} onChange={(variable) => onChange({ ...node, variable })} mono />
      <NodeSelect label="Next" value={node.next} onChange={(next) => onChange({ ...node, next })} nodeIds={nodeIds} />

      <div className="rounded-lg border border-white/10 bg-white/[0.02] p-2.5">
        <div className="mb-2 flex items-center justify-between">
          <p className="text-[11px] font-medium uppercase tracking-wider text-zinc-500">Validation</p>
          <button
            type="button"
            onClick={() => onChange({ ...node, validate: validate ? undefined : { type: "text" } })}
            className="rounded px-1.5 text-xs text-zinc-500 hover:text-emerald-300"
          >
            {validate ? "remove" : "add"}
          </button>
        </div>
        {validate && (
          <div className="space-y-2">
            <SelectField
              label="Type"
              value={validate.type}
              onChange={(type) => onChange({ ...node, validate: { ...validate, type: type as ValidationKind } })}
              options={VALIDATION_KINDS}
            />
            <div className="grid grid-cols-2 gap-2">
              <NumberField label="Min" value={validate.min} onChange={(min) => onChange({ ...node, validate: { ...validate, min } })} />
              <NumberField label="Max" value={validate.max} onChange={(max) => onChange({ ...node, validate: { ...validate, max } })} />
              <NumberField label="Min len" value={validate.minLength} onChange={(minLength) => onChange({ ...node, validate: { ...validate, minLength } })} />
              <NumberField label="Max len" value={validate.maxLength} onChange={(maxLength) => onChange({ ...node, validate: { ...validate, maxLength } })} />
            </div>
            <TextField label="Pattern" value={validate.pattern ?? ""} onChange={(pattern) => onChange({ ...node, validate: { ...validate, pattern: pattern || undefined } })} mono />
            <TextField label="Message" value={validate.message ?? ""} onChange={(message) => onChange({ ...node, validate: { ...validate, message: message || undefined } })} />
          </div>
        )}
      </div>

      {node.onInvalid && (
        <RecoveryEditor label="onInvalid" recovery={node.onInvalid} nodeIds={nodeIds} onChange={(onInvalid) => onChange({ ...node, onInvalid })} />
      )}
      {node.onTimeout && (
        <RecoveryEditor label="onTimeout" recovery={node.onTimeout} nodeIds={nodeIds} onChange={(onTimeout) => onChange({ ...node, onTimeout })} />
      )}
    </div>
  );
}

function ActionEditor({
  node,
  nodeIds,
  onChange,
}: {
  node: Extract<Node, { type: "action" }>;
  nodeIds: string[];
  onChange: (n: Node) => void;
}) {
  return (
    <div className="space-y-3">
      <MapEditor
        label="Set"
        map={Object.fromEntries(Object.entries(node.set ?? {}).map(([k, v]) => [k, String(v)]))}
        onChange={(m) => onChange({ ...node, set: m })}
        monoValue
      />
      <MapEditor
        label="Compute"
        map={node.compute ?? {}}
        onChange={(compute) => onChange({ ...node, compute })}
        valuePlaceholder="expr"
        monoValue
      />
      <NodeSelect label="Next" value={node.next} onChange={(next) => onChange({ ...node, next })} nodeIds={nodeIds} />
    </div>
  );
}

function EndEditor({
  node,
  onChange,
}: {
  node: Extract<Node, { type: "end" }>;
  onChange: (n: Node) => void;
}) {
  return (
    <div className="space-y-3">
      <TextArea label="Text" value={node.text} onChange={(text) => onChange({ ...node, text })} />
      {node.payments?.mpesa && (
        <div className="rounded-lg border border-white/10 bg-white/[0.02] p-2.5">
          <p className="mb-2 text-[11px] font-medium uppercase tracking-wider text-zinc-500">
            M-Pesa STK push
          </p>
          <div className="grid grid-cols-2 gap-2">
            <TextField label="Short code" value={node.payments.mpesa.shortCode} onChange={(shortCode) => onChange({ ...node, payments: { mpesa: { ...node.payments!.mpesa!, shortCode } } })} mono />
            <TextField label="Amount expr" value={node.payments.mpesa.amountExpr} onChange={(amountExpr) => onChange({ ...node, payments: { mpesa: { ...node.payments!.mpesa!, amountExpr } } })} mono />
            <TextField label="Phone expr" value={node.payments.mpesa.phoneExpr} onChange={(phoneExpr) => onChange({ ...node, payments: { mpesa: { ...node.payments!.mpesa!, phoneExpr } } })} mono />
            <TextField label="Account ref" value={node.payments.mpesa.accountRef ?? ""} onChange={(accountRef) => onChange({ ...node, payments: { mpesa: { ...node.payments!.mpesa!, accountRef: accountRef || undefined } } })} mono />
          </div>
        </div>
      )}
    </div>
  );
}

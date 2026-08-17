/**
 * Live validation for the flow builder: formal JSON Schema conformance (ajv,
 * against docs/menu-schema.schema.json) plus semantic checks the schema can't
 * express — dangling targets and cycles without an input node — mirroring the
 * engine's deploy-time validator (engine/crates/engine/src/schema/validate.rs).
 */

import Ajv, { type ErrorObject } from "ajv";
import schemaJson from "./menu-schema.schema.json";
import type { Flow } from "./types";
import { outTargets } from "./types";

export interface ValidationIssue {
  /** JSON-pointer-style path ("" = document root). */
  path: string;
  message: string;
  kind: "schema" | "semantic";
}

const ajv = new Ajv({ allErrors: true, strict: false });
const validateDoc = ajv.compile(schemaJson as object);

/** Validate a full FlowDocument against the formal JSON Schema. */
export function validateDocument(doc: unknown): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  if (!validateDoc(doc)) {
    for (const err of validateDoc.errors ?? []) {
      issues.push(schemaIssue(err));
    }
  }
  const flow = (doc as { flow?: Flow } | null)?.flow;
  if (flow) issues.push(...validateFlowSemantics(flow));
  return issues;
}

/** Validate the loaded flow for semantic (non-JSON-Schema) problems. */
export function validateFlowSemantics(flow: Flow): ValidationIssue[] {
  const issues: ValidationIssue[] = [];

  if (!flow.nodes[flow.start]) {
    issues.push({
      path: "/flow/start",
      message: `start node '${flow.start}' does not exist`,
      kind: "semantic",
    });
  }

  // Dangling references.
  for (const [id, node] of Object.entries(flow.nodes)) {
    for (const target of outTargets(node)) {
      if (!flow.nodes[target]) {
        issues.push({
          path: `/flow/nodes/${id}`,
          message: `node '${id}' references missing node '${target}'`,
          kind: "semantic",
        });
      }
    }
  }

  // Cycles with no input node can never progress (spec §1, §13).
  issues.push(...findProgresslessCycles(flow));

  return issues;
}

function schemaIssue(err: ErrorObject): ValidationIssue {
  const path = err.instancePath || "";
  const pretty = err.message ?? "invalid";
  // Tidy common error messages: "must NOT have additional properties" →
  // "unexpected property 'foo'".
  if (err.keyword === "additionalProperties") {
    const prop = (err.params as { additionalProperty?: string }).additionalProperty;
    return { path, message: `unexpected property '${prop}'`, kind: "schema" };
  }
  return { path, message: pretty, kind: "schema" };
}

/**
 * Find cycles (via DFS back-edges) whose members contain no `input` node.
 * Returns one issue per distinct offending cycle.
 */
function findProgresslessCycles(flow: Flow): ValidationIssue[] {
  const nodes = flow.nodes;
  const WHITE = 0;
  const GRAY = 1;
  const BLACK = 2;
  const color = new Map<string, number>();
  const stack: string[] = [];
  const seen = new Set<string>();

  const hasInput = (ids: string[]) => ids.some((id) => nodes[id]?.type === "input");

  const key = (ids: string[]) => [...ids].sort().join("|");

  const dfs = (id: string) => {
    color.set(id, GRAY);
    stack.push(id);
    const node = nodes[id];
    if (node) {
      for (const t of outTargets(node)) {
        if (!nodes[t]) continue;
        const c = color.get(t) ?? WHITE;
        if (c === GRAY) {
          // Back edge t → cycle = stack[stack.indexOf(t)..end]
          const start = stack.indexOf(t);
          const cycle = stack.slice(start);
          if (cycle.length > 0 && !hasInput(cycle)) {
            seen.add(key(cycle));
          }
        } else if (c === WHITE) {
          dfs(t);
        }
      }
    }
    stack.pop();
    color.set(id, BLACK);
  };

  for (const id of Object.keys(nodes)) {
    if ((color.get(id) ?? WHITE) === WHITE) dfs(id);
  }

  return [...seen].map((k) => ({
    path: `/flow/nodes`,
    message: `cycle with no input node can never progress: ${k.split("|").join(" → ")}`,
    kind: "semantic" as const,
  }));
}

/** Count issues by kind. */
export function summarizeIssues(issues: ValidationIssue[]) {
  const schema = issues.filter((i) => i.kind === "schema").length;
  const semantic = issues.filter((i) => i.kind === "semantic").length;
  return { total: issues.length, schema, semantic };
}

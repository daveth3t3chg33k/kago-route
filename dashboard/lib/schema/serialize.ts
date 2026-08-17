/**
 * Serialization for the builder's output panel: copyable YAML and JSON that
 * round-trip through the engine's parser. The engine reads YAML with serde_yaml
 * and JSON with serde_json using the same camelCase field names these types
 * use, so the output is directly deployable via FLOW_SCHEMA_PATH.
 */

import { stringify as yamlStringify } from "yaml";
import type { FlowDocument } from "./types";
import { DSL_IDENTIFIER } from "./types";

/** Strip undefined values (JSON has no undefined) before emitting. */
function clean<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export function toJson(doc: FlowDocument): string {
  return JSON.stringify(clean(doc), null, 2);
}

export function toYaml(doc: FlowDocument): string {
  return yamlStringify(clean(doc), { indent: 2 });
}

/** Build a minimal valid document (start menu + farewell end). */
export function blankDocument(name = "my-flow"): FlowDocument {
  return {
    schema: DSL_IDENTIFIER,
    flow: {
      id: name,
      name,
      version: 1,
      start: "welcome",
      timeouts: { session: 120, step: 20 },
      nodes: {
        welcome: {
          type: "menu",
          text: "Welcome to KagoRoute demo!\n1. Continue",
          options: { "1": { goto: "farewell" } },
        },
        farewell: {
          type: "end",
          text: "Thank you. Goodbye.",
        },
      },
    },
  };
}

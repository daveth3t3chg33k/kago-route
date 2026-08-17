/**
 * TypeScript types mirroring the KagoRoute menu DSL (docs/ussd-menu-schema.md
 * Appendix B and engine/crates/engine/src/schema/mod.rs). Field names use the
 * wire format (camelCase) exactly as parsed/serialized by the engine.
 */

export const DSL_IDENTIFIER = "kagoroute/menu/1.0";

export interface FlowDocument {
  schema: string;
  flow: Flow;
}

export interface Flow {
  id: string;
  name: string;
  description?: string;
  version: number;
  start: string;
  timeouts?: Timeouts;
  variables?: VariableDecl[];
  webhooks?: Webhooks;
  nodes: Record<string, Node>;
}

export interface Timeouts {
  session?: number;
  step?: number;
}

export interface VariableDecl {
  name: string;
  type?: "int" | "float" | "text" | "phone" | "amount";
  source?: "caller";
}

export interface Webhooks {
  onComplete?: Webhook;
}

export interface Webhook {
  url: string;
  secret?: string;
  events?: Array<"complete" | "invalid" | "timeout" | "payment.result">;
}

export type NodeType = "menu" | "input" | "action" | "end";

export interface BaseNode {
  type: NodeType;
}

export interface MenuNode extends BaseNode {
  type: "menu";
  text: string;
  options: Record<string, OptionValue>;
  onInvalid?: Recovery;
  onTimeout?: Recovery;
}

export interface InputNode extends BaseNode {
  type: "input";
  prompt: string;
  variable: string;
  validate?: Validation;
  onInvalid?: Recovery;
  onTimeout?: Recovery;
  next: string;
}

export interface ActionNode extends BaseNode {
  type: "action";
  set?: Record<string, unknown>;
  compute?: Record<string, string>;
  next: string;
}

export interface EndNode extends BaseNode {
  type: "end";
  text: string;
  payments?: Payments;
  onTimeout?: Recovery;
}

export type Node = MenuNode | InputNode | ActionNode | EndNode;

export type OptionValue = Branch | Branch[];

export interface Branch {
  when?: Condition | Condition[];
  set?: Record<string, unknown>;
  compute?: Record<string, string>;
  goto: string;
}

export type Op =
  | "eq"
  | "neq"
  | "gt"
  | "gte"
  | "lt"
  | "lte"
  | "contains"
  | "startsWith"
  | "matches"
  | "isSet"
  | "in";

export interface Condition {
  var: string;
  op: Op;
  value?: unknown;
}

export type ValidationKind =
  | "int"
  | "float"
  | "text"
  | "phone"
  | "amount"
  | "option";

export interface Validation {
  type: ValidationKind;
  min?: number;
  max?: number;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
  options?: string[];
  message?: string;
}

export interface Recovery {
  text?: string;
  goto: string;
}

export interface Payments {
  mpesa?: MpesaStkPush;
}

export interface MpesaStkPush {
  shortCode: string;
  amountExpr: string;
  phoneExpr: string;
  accountRef?: string;
  transactionDesc?: string;
}

/** Normalize an OptionValue (Branch | Branch[]) to an array. */
export function branchesOf(value: OptionValue): Branch[] {
  return Array.isArray(value) ? value : [value];
}

/** Every node id referenced from `node` (goto/next/onInvalid/onTimeout). */
export function outTargets(node: Node): string[] {
  const targets: string[] = [];
  const push = (s?: string) => {
    if (s) targets.push(s);
  };
  switch (node.type) {
    case "menu":
      for (const opt of Object.values(node.options)) {
        for (const b of branchesOf(opt)) push(b.goto);
      }
      push(node.onInvalid?.goto);
      push(node.onTimeout?.goto);
      break;
    case "input":
      push(node.next);
      push(node.onInvalid?.goto);
      push(node.onTimeout?.goto);
      break;
    case "action":
      push(node.next);
      break;
    case "end":
      push(node.onTimeout?.goto);
      break;
  }
  return targets;
}

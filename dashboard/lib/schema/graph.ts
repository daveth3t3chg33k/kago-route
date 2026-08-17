/**
 * Graph helpers for the visual builder: layered auto-layout (BFS depth from
 * the start node) and edge enumeration (which node connects to which, with
 * option-digit labels).
 */

import type { Node } from "./types";
import { branchesOf } from "./types";

export const NODE_W = 260;
export const NODE_H = 150;
const GAP_X = 90;
const GAP_Y = 44;

export interface GraphEdge {
  from: string;
  to: string;
  /** Option digit / semantic label (e.g. "1", "2", "next"). */
  label?: string;
}

/** All edges in the graph, with labels for menu options. */
export function collectEdges(nodes: Record<string, Node>): GraphEdge[] {
  const edges: GraphEdge[] = [];
  for (const [id, node] of Object.entries(nodes)) {
    switch (node.type) {
      case "menu":
        for (const [digit, opt] of Object.entries(node.options)) {
          for (const b of branchesOf(opt)) {
            edges.push({ from: id, to: b.goto, label: digit });
          }
        }
        if (node.onInvalid) edges.push({ from: id, to: node.onInvalid.goto, label: "invalid" });
        if (node.onTimeout) edges.push({ from: id, to: node.onTimeout.goto, label: "timeout" });
        break;
      case "input":
        edges.push({ from: id, to: node.next, label: "next" });
        if (node.onInvalid) edges.push({ from: id, to: node.onInvalid.goto, label: "invalid" });
        if (node.onTimeout) edges.push({ from: id, to: node.onTimeout.goto, label: "timeout" });
        break;
      case "action":
        edges.push({ from: id, to: node.next, label: "next" });
        break;
      case "end":
        if (node.onTimeout) edges.push({ from: id, to: node.onTimeout.goto, label: "timeout" });
        break;
    }
  }
  return edges;
}

export interface Point {
  x: number;
  y: number;
}

/**
 * Layered auto-layout: BFS depth from `start` → column; order within a layer
 * → row. Nodes unreachable from `start` are laid out below the main flow.
 */
export function autoLayout(
  nodes: Record<string, Node>,
  start: string,
): Record<string, Point> {
  const edges = collectEdges(nodes);
  const depth = new Map<string, number>();
  const order: string[] = [];

  // BFS from start.
  const queue: string[] = [start];
  depth.set(start, 0);
  while (queue.length) {
    const cur = queue.shift()!;
    order.push(cur);
    for (const e of edges) {
      if (e.from === cur && !depth.has(e.to) && nodes[e.to]) {
        depth.set(e.to, (depth.get(cur) ?? 0) + 1);
        queue.push(e.to);
      }
    }
  }

  const maxDepth = Math.max(0, ...depth.values());
  // Unreachable nodes (dangling targets or orphans) → deepest column.
  for (const id of Object.keys(nodes)) {
    if (!depth.has(id)) {
      depth.set(id, maxDepth + 1);
      order.push(id);
    }
  }

  // Group by depth, preserving BFS/insertion order within each layer.
  const layers = new Map<number, string[]>();
  for (const id of order) {
    const d = depth.get(id) ?? maxDepth + 1;
    if (!layers.has(d)) layers.set(d, []);
    layers.get(d)!.push(id);
  }

  const positions: Record<string, Point> = {};
  for (const [d, ids] of layers) {
    // Center each layer vertically around y=0; drag offsets can push it down.
    const center = ((ids.length - 1) * (NODE_H + GAP_Y)) / 2;
    ids.forEach((id, i) => {
      positions[id] = {
        x: d * (NODE_W + GAP_X),
        y: i * (NODE_H + GAP_Y) - center + 40,
      };
    });
  }

  for (const id of Object.keys(nodes)) {
    if (!positions[id]) positions[id] = { x: 0, y: 0 };
  }
  return positions;
}

/** Canvas size in px for the scroll area, given a layout. */
export function canvasSize(positions: Record<string, Point>): { width: number; height: number } {
  let maxX = 0;
  let maxY = 0;
  for (const p of Object.values(positions)) {
    maxX = Math.max(maxX, p.x + NODE_W);
    maxY = Math.max(maxY, p.y + NODE_H);
  }
  return { width: maxX + 120, height: maxY + 120 };
}

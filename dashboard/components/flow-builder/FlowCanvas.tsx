"use client";

import { useCallback, useRef, useState } from "react";
import type { Node } from "@/lib/schema/types";
import { NODE_H, NODE_W, canvasSize, collectEdges, type Point } from "@/lib/schema/graph";
import { NodeCard } from "./NodeCard";

function edgePath(from: Point, to: Point): string {
  const x1 = from.x + NODE_W;
  const y1 = from.y + NODE_H / 2;
  const x2 = to.x;
  const y2 = to.y + NODE_H / 2;
  const dx = Math.max(36, (x2 - x1) / 2);
  return `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
}

export function FlowCanvas({
  nodes,
  positions,
  selectedId,
  onSelect,
  onMove,
}: {
  nodes: Record<string, Node>;
  positions: Record<string, Point>;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onMove: (id: string, pos: Point) => void;
}) {
  const edges = collectEdges(nodes);
  const size = canvasSize(positions);
  const [drag, setDrag] = useState<{ id: string; dx: number; dy: number } | null>(null);
  const startPosRef = useRef<Point | null>(null);

  const onPointerDown = useCallback(
    (e: React.PointerEvent, id: string) => {
      e.stopPropagation();
      onSelect(id);
      startPosRef.current = positions[id] ?? { x: 0, y: 0 };
      setDrag({ id, dx: e.clientX, dy: e.clientY });
      (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
    },
    [onSelect, positions],
  );

  const onPointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!drag || !startPosRef.current) return;
      const nx = startPosRef.current.x + (e.clientX - drag.dx);
      const ny = startPosRef.current.y + (e.clientY - drag.dy);
      onMove(drag.id, { x: Math.max(0, nx), y: Math.max(0, ny) });
    },
    [drag, onMove],
  );

  const onPointerUp = useCallback(() => {
    setDrag(null);
    startPosRef.current = null;
  }, []);

  return (
    <div
      className="relative h-full w-full overflow-auto rounded-xl border border-white/10 bg-[#0a100e]"
      style={{
        backgroundImage:
          "radial-gradient(circle at 1px 1px, rgba(255,255,255,0.045) 1px, transparent 0)",
        backgroundSize: "24px 24px",
      }}
      onClick={() => onSelect(null)}
    >
      <div
        className="relative"
        style={{ width: size.width, height: size.height }}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerLeave={onPointerUp}
      >
        {/* Edges */}
        <svg
          className="pointer-events-none absolute inset-0"
          width={size.width}
          height={size.height}
        >
          <defs>
            <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#34d399" />
            </marker>
          </defs>
          {edges.map((e, i) => {
            const from = positions[e.from];
            const to = positions[e.to];
            if (!from || !to) return null;
            const label = e.label;
            const midX = (from.x + to.x + NODE_W) / 2;
            const midY = (from.y + to.y + NODE_H) / 2;
            return (
              <g key={`${e.from}-${e.to}-${i}`}>
                <path
                  d={edgePath(from, to)}
                  fill="none"
                  stroke="#34d399"
                  strokeOpacity={0.35}
                  strokeWidth={1.5}
                  markerEnd="url(#arrow)"
                />
                {label ? (
                  <text
                    x={midX}
                    y={midY - 6}
                    textAnchor="middle"
                    className="fill-emerald-300"
                    fontSize="10"
                    fontFamily="ui-monospace, monospace"
                  >
                    {label}
                  </text>
                ) : null}
              </g>
            );
          })}
        </svg>

        {/* Nodes */}
        {Object.entries(nodes).map(([id, node]) => {
          const pos = positions[id] ?? { x: 0, y: 0 };
          return (
            <div
              key={id}
              className="absolute cursor-grab active:cursor-grabbing"
              style={{
                left: pos.x,
                top: pos.y,
                width: NODE_W,
                touchAction: "none",
                zIndex: selectedId === id ? 20 : 10,
              }}
              onPointerDown={(e) => onPointerDown(e, id)}
            >
              <NodeCard id={id} node={node} selected={selectedId === id} onSelect={onSelect} />
            </div>
          );
        })}
      </div>

      {Object.keys(nodes).length === 0 && (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
          <p className="text-sm text-zinc-600">No nodes yet — add one from the toolbar.</p>
        </div>
      )}
    </div>
  );
}

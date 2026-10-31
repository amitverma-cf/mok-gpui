import { GHOST, HEIGHT, INK, WIDTH, runs } from "./wordmark";

const rects = (rows: string[]) => runs(rows).map((r) => <rect key={`${r.x}-${r.y}`} x={r.x} y={r.y} width={r.width} height={1} opacity={r.opacity} />);

/** MOK in torn block characters; the accent copy of the torn rows jitters unless motion is reduced. */
export function Mark() {
  return (
    <svg className="mark" viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="img" aria-label="Mok" shapeRendering="crispEdges">
      <g className="ghost">{rects(GHOST)}</g>
      <g className="ink">{rects(INK)}</g>
    </svg>
  );
}

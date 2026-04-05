/** Command payload for drawing a debug line gizmo. */
export interface CmdGizmoDrawLineArgs {
  start: [number, number, number];
  end: [number, number, number];
  color: [number, number, number, number];
  thickness?: number;
}

/** Command payload for drawing a debug AABB gizmo. */
export interface CmdGizmoDrawAabbArgs {
  min: [number, number, number];
  max: [number, number, number];
  color: [number, number, number, number];
  thickness?: number;
}

/** Command payload for drawing a debug polyline gizmo. */
export interface CmdGizmoDrawPolylineArgs {
  points: [number, number, number][];
  color: [number, number, number, number];
  closed?: boolean;
  thickness?: number;
}

/** Result payload for gizmo draw commands. */
export interface CmdResultGizmoDraw {
  status: number;
}

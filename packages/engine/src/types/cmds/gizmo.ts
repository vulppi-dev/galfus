import type { vec3, vec4 } from 'gl-matrix';
/** Command payload for drawing a debug line gizmo. */
export interface CmdGizmoDrawLineArgs {
  start: vec3;
  end: vec3;
  color: vec4;
  thickness?: number;
}

/** Command payload for drawing a debug AABB gizmo. */
export interface CmdGizmoDrawAabbArgs {
  min: vec3;
  max: vec3;
  color: vec4;
  thickness?: number;
}

/** Command payload for drawing a debug polyline gizmo. */
export interface CmdGizmoDrawPolylineArgs {
  points: vec3[];
  color: vec4;
  closed?: boolean;
  thickness?: number;
}

/** Result payload for gizmo draw commands. */
export interface CmdResultGizmoDraw {
  status: number;
}

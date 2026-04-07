import { mat4, quat, vec3, type ReadonlyQuat, type ReadonlyVec3 } from 'gl-matrix';

export function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export function makeLookRotation(
  out: quat,
  eye: ReadonlyVec3,
  target: ReadonlyVec3,
  up: ReadonlyVec3 = [0, 1, 0]
): quat {
  const view = mat4.create();
  mat4.targetTo(view, eye, target, up);
  mat4.getRotation(out, view);
  return quat.normalize(out, out);
}

export function slerpArc(
  out: quat,
  from: ReadonlyQuat,
  to: ReadonlyQuat,
  t: number,
  longArc: boolean
): quat {
  const x0 = from[0] ?? 0;
  const y0 = from[1] ?? 0;
  const z0 = from[2] ?? 0;
  const w0 = from[3] ?? 1;

  let x1 = to[0] ?? 0;
  let y1 = to[1] ?? 0;
  let z1 = to[2] ?? 0;
  let w1 = to[3] ?? 1;

  let cos = x0 * x1 + y0 * y1 + z0 * z1 + w0 * w1;
  if ((!longArc && cos < 0) || (longArc && cos > 0)) {
    x1 = -x1;
    y1 = -y1;
    z1 = -z1;
    w1 = -w1;
    cos = -cos;
  }

  if (1 - Math.abs(cos) < 1e-6) {
    out[0] = x0 + t * (x1 - x0);
    out[1] = y0 + t * (y1 - y0);
    out[2] = z0 + t * (z1 - z0);
    out[3] = w0 + t * (w1 - w0);
    return quat.normalize(out, out);
  }

  const omega = Math.acos(cos);
  const sinOmega = Math.sin(omega);
  const s0 = Math.sin((1 - t) * omega) / sinOmega;
  const s1 = Math.sin(t * omega) / sinOmega;

  out[0] = s0 * x0 + s1 * x1;
  out[1] = s0 * y0 + s1 * y1;
  out[2] = s0 * z0 + s1 * z1;
  out[3] = s0 * w0 + s1 * w1;
  return quat.normalize(out, out);
}

export function smoothStepAlpha(weight: number, dtSeconds: number): number {
  const speed = Math.max(0, Math.abs(weight));
  return 1 - Math.exp(-speed * dtSeconds * 10);
}

export function sphericalToCartesian(out: vec3, yaw: number, pitch: number, radius: number): vec3 {
  const cp = Math.cos(pitch);
  out[0] = Math.sin(yaw) * cp * radius;
  out[1] = Math.sin(pitch) * radius;
  out[2] = Math.cos(yaw) * cp * radius;
  return out;
}

export function localBasisFromQuat(
  rotation: ReadonlyQuat,
  outForward: vec3,
  outRight: vec3,
  outUp: vec3
): void {
  vec3.transformQuat(outForward, [0, 0, -1], rotation);
  vec3.normalize(outForward, outForward);

  vec3.transformQuat(outRight, [1, 0, 0], rotation);
  vec3.normalize(outRight, outRight);

  vec3.transformQuat(outUp, [0, 1, 0], rotation);
  vec3.normalize(outUp, outUp);
}

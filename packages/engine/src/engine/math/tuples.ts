import { mat4, quat, vec2, vec3, vec4 } from 'gl-matrix';
import type { Matrix4 } from '../../types/cmds/camera';

export function createVec2Tuple(x = 0, y = 0): [number, number] {
  const value = vec2.fromValues(x, y);
  return [value[0], value[1]];
}

export function createVec3Tuple(x = 0, y = 0, z = 0): [number, number, number] {
  const value = vec3.fromValues(x, y, z);
  return [value[0], value[1], value[2]];
}

export function createVec4Tuple(x = 0, y = 0, z = 0, w = 0): [number, number, number, number] {
  const value = vec4.fromValues(x, y, z, w);
  return [value[0], value[1], value[2], value[3]];
}

export function createQuatTuple(x = 0, y = 0, z = 0, w = 1): [number, number, number, number] {
  const value = quat.fromValues(x, y, z, w);
  return [value[0], value[1], value[2], value[3]];
}

export function createMat4Tuple(): Matrix4 {
  const value = mat4.create();
  return [
    value[0],
    value[1],
    value[2],
    value[3],
    value[4],
    value[5],
    value[6],
    value[7],
    value[8],
    value[9],
    value[10],
    value[11],
    value[12],
    value[13],
    value[14],
    value[15]
  ];
}

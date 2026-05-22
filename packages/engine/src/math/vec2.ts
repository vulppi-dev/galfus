import type { IndexedCollection } from './common';
import type { ReadonlyMat2 } from './mat2';
import type { ReadonlyMat2d } from './mat2d';
import type { ReadonlyMat3 } from './mat3';
import type { ReadonlyMat4 } from './mat4';
import type { vec3 } from './vec3';
import * as vectorMath from './common';

export type vec2 = [number, number] | IndexedCollection;
export type Vec2 = vec2;
export type ReadonlyVec2 = readonly [number, number] | IndexedCollection;

/**
 * 2 Dimensional Vector
 * @module vec2
 */

/**
 * Creates a new, empty vec2
 *
 * @returns {vec2} a new 2D vector
 */
export function create(): vec2 {
  let out = vectorMath.createArray(2) as Vec2;

  return out as Vec2;
}

/**
 * Creates a new vec2 initialized with values from an existing vector
 *
 * @param {ReadonlyVec2} a vector to clone
 * @returns {vec2} a new 2D vector
 */
export function clone(a: ReadonlyVec2): vec2 {
  let out = vectorMath.createArray(2) as Vec2;
  out[0] = a[0];
  out[1] = a[1];
  return out as Vec2;
}

/**
 * Creates a new vec2 initialized with the given values
 *
 * @param {Number} x X component
 * @param {Number} y Y component
 * @returns {vec2} a new 2D vector
 */
export function fromValues(x: number, y: number): vec2 {
  let out = vectorMath.createArray(2) as Vec2;
  out[0] = x;
  out[1] = y;
  return out as Vec2;
}

/**
 * Copy the values from one vec2 to another
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the source vector
 * @returns {vec2} out
 */
export function copy(out: vec2, a: ReadonlyVec2): vec2 {
  out[0] = a[0];
  out[1] = a[1];
  return out as Vec2;
}

/**
 * Set the components of a vec2 to the given values
 *
 * @param {vec2} out the receiving vector
 * @param {Number} x X component
 * @param {Number} y Y component
 * @returns {vec2} out
 */
export function set(out: vec2, x: number, y: number): vec2 {
  out[0] = x;
  out[1] = y;
  return out as Vec2;
}

/**
 * Adds two vec2's
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {vec2} out
 */
export function add(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2): vec2 {
  out[0] = a[0] + b[0];
  out[1] = a[1] + b[1];
  return out as Vec2;
}

/**
 * Subtracts vector b from vector a
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {vec2} out
 */
export function subtract(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2): vec2 {
  out[0] = a[0] - b[0];
  out[1] = a[1] - b[1];
  return out as Vec2;
}

/**
 * Multiplies two vec2's
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {vec2} out
 */
export function multiply(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2): vec2 {
  out[0] = a[0] * b[0];
  out[1] = a[1] * b[1];
  return out as Vec2;
}

/**
 * Divides two vec2's
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {vec2} out
 */
export function divide(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2): vec2 {
  out[0] = a[0] / b[0];
  out[1] = a[1] / b[1];
  return out as Vec2;
}

/**
 * Math.ceil the components of a vec2
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a vector to ceil
 * @returns {vec2} out
 */
export function ceil(out: vec2, a: ReadonlyVec2): vec2 {
  out[0] = Math.ceil(a[0]);
  out[1] = Math.ceil(a[1]);
  return out as Vec2;
}

/**
 * Math.floor the components of a vec2
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a vector to floor
 * @returns {vec2} out
 */
export function floor(out: vec2, a: ReadonlyVec2): vec2 {
  out[0] = Math.floor(a[0]);
  out[1] = Math.floor(a[1]);
  return out as Vec2;
}

/**
 * Returns the minimum of two vec2's
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {vec2} out
 */
export function min(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2): vec2 {
  out[0] = Math.min(a[0], b[0]);
  out[1] = Math.min(a[1], b[1]);
  return out as Vec2;
}

/**
 * Returns the maximum of two vec2's
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {vec2} out
 */
export function max(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2): vec2 {
  out[0] = Math.max(a[0], b[0]);
  out[1] = Math.max(a[1], b[1]);
  return out as Vec2;
}

/**
 * symmetric round the components of a vec2
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a vector to round
 * @returns {vec2} out
 */
export function round(out: vec2, a: ReadonlyVec2): vec2 {
  out[0] = vectorMath.round(a[0]);
  out[1] = vectorMath.round(a[1]);
  return out as Vec2;
}

/**
 * Scales a vec2 by a scalar number
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the vector to scale
 * @param {Number} b amount to scale the vector by
 * @returns {vec2} out
 */
export function scale(out: vec2, a: ReadonlyVec2, b: number): vec2 {
  out[0] = a[0] * b;
  out[1] = a[1] * b;
  return out as Vec2;
}

/**
 * Adds two vec2's after scaling the second operand by a scalar value
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @param {Number} scale the amount to scale b by before adding
 * @returns {vec2} out
 */
export function scaleAndAdd(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2, scale: number): vec2 {
  out[0] = a[0] + b[0] * scale;
  out[1] = a[1] + b[1] * scale;
  return out as Vec2;
}

/**
 * Calculates the euclidian distance between two vec2's
 *
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {Number} distance between a and b
 */
export function distance(a: ReadonlyVec2, b: ReadonlyVec2): number {
  var x = b[0] - a[0],
    y = b[1] - a[1];
  return Math.sqrt(x * x + y * y);
}

/**
 * Calculates the squared euclidian distance between two vec2's
 *
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {Number} squared distance between a and b
 */
export function squaredDistance(a: ReadonlyVec2, b: ReadonlyVec2): number {
  var x = b[0] - a[0],
    y = b[1] - a[1];
  return x * x + y * y;
}

/**
 * Calculates the length of a vec2
 *
 * @param {ReadonlyVec2} a vector to calculate length of
 * @returns {Number} length of a
 */
export function length(a: ReadonlyVec2): number {
  var x = a[0],
    y = a[1];
  return Math.sqrt(x * x + y * y);
}

/**
 * Calculates the squared length of a vec2
 *
 * @param {ReadonlyVec2} a vector to calculate squared length of
 * @returns {Number} squared length of a
 */
export function squaredLength(a: ReadonlyVec2): number {
  var x = a[0],
    y = a[1];
  return x * x + y * y;
}

/**
 * Negates the components of a vec2
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a vector to negate
 * @returns {vec2} out
 */
export function negate(out: vec2, a: ReadonlyVec2): vec2 {
  out[0] = -a[0];
  out[1] = -a[1];
  return out as Vec2;
}

/**
 * Returns the inverse of the components of a vec2
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a vector to invert
 * @returns {vec2} out
 */
export function inverse(out: vec2, a: ReadonlyVec2): vec2 {
  out[0] = 1.0 / a[0];
  out[1] = 1.0 / a[1];
  return out as Vec2;
}

/**
 * Normalize a vec2
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a vector to normalize
 * @returns {vec2} out
 */
export function normalize(out: vec2, a: ReadonlyVec2): vec2 {
  var x = a[0],
    y = a[1];
  var len = x * x + y * y;
  if (len > 0) {
    //TODO: evaluate use of glm_invsqrt here?
    len = 1 / Math.sqrt(len);
  }
  out[0] = a[0] * len;
  out[1] = a[1] * len;
  return out as Vec2;
}

/**
 * Calculates the dot product of two vec2's
 *
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {Number} dot product of a and b
 */
export function dot(a: ReadonlyVec2, b: ReadonlyVec2): number {
  return a[0] * b[0] + a[1] * b[1];
}

/**
 * Computes the cross product of two vec2's
 * Note that the cross product must by definition produce a 3D vector
 *
 * @param {vec3} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @returns {vec3} out
 */
export function cross(out: vec3, a: ReadonlyVec2, b: ReadonlyVec2): vec3 {
  var z = a[0] * b[1] - a[1] * b[0];
  out[0] = out[1] = 0;
  out[2] = z;
  return out;
}

/**
 * Performs a linear interpolation between two vec2's
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the first operand
 * @param {ReadonlyVec2} b the second operand
 * @param {Number} t interpolation amount, in the range [0-1], between the two inputs
 * @returns {vec2} out
 */
export function lerp(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2, t: number): vec2 {
  var ax = a[0],
    ay = a[1];
  out[0] = ax + t * (b[0] - ax);
  out[1] = ay + t * (b[1] - ay);
  return out as Vec2;
}

/**
 * Generates a random vector with the given scale
 *
 * @param {vec2} out the receiving vector
 * @param {Number} [scale] Length of the resulting vector. If omitted, a unit vector will be returned
 * @returns {vec2} out
 */
export function random(out: vec2, scale: number): vec2 {
  scale = scale === undefined ? 1.0 : scale;
  var r = vectorMath.RANDOM() * 2.0 * Math.PI;
  out[0] = Math.cos(r) * scale;
  out[1] = Math.sin(r) * scale;
  return out as Vec2;
}

/**
 * Transforms the vec2 with a mat2
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the vector to transform
 * @param {ReadonlyMat2} m matrix to transform with
 * @returns {vec2} out
 */
export function transformMat2(out: vec2, a: ReadonlyVec2, m: ReadonlyMat2): vec2 {
  var x = a[0],
    y = a[1];
  out[0] = m[0] * x + m[2] * y;
  out[1] = m[1] * x + m[3] * y;
  return out as Vec2;
}

/**
 * Transforms the vec2 with a mat2d
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the vector to transform
 * @param {ReadonlyMat2d} m matrix to transform with
 * @returns {vec2} out
 */
export function transformMat2d(out: vec2, a: ReadonlyVec2, m: ReadonlyMat2d): vec2 {
  var x = a[0],
    y = a[1];
  out[0] = m[0] * x + m[2] * y + m[4];
  out[1] = m[1] * x + m[3] * y + m[5];
  return out as Vec2;
}

/**
 * Transforms the vec2 with a mat3
 * 3rd vector component is implicitly '1'
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the vector to transform
 * @param {ReadonlyMat3} m matrix to transform with
 * @returns {vec2} out
 */
export function transformMat3(out: vec2, a: ReadonlyVec2, m: ReadonlyMat3): vec2 {
  var x = a[0],
    y = a[1];
  out[0] = m[0] * x + m[3] * y + m[6];
  out[1] = m[1] * x + m[4] * y + m[7];
  return out as Vec2;
}

/**
 * Transforms the vec2 with a mat4
 * 3rd vector component is implicitly '0'
 * 4th vector component is implicitly '1'
 *
 * @param {vec2} out the receiving vector
 * @param {ReadonlyVec2} a the vector to transform
 * @param {ReadonlyMat4} m matrix to transform with
 * @returns {vec2} out
 */
export function transformMat4(out: vec2, a: ReadonlyVec2, m: ReadonlyMat4): vec2 {
  let x = a[0];
  let y = a[1];
  out[0] = m[0] * x + m[4] * y + m[12];
  out[1] = m[1] * x + m[5] * y + m[13];
  return out as Vec2;
}

/**
 * Rotate a 2D vector
 * @param {vec2} out The receiving vec2
 * @param {ReadonlyVec2} a The vec2 point to rotate
 * @param {ReadonlyVec2} b The origin of the rotation
 * @param {Number} rad The angle of rotation in radians
 * @returns {vec2} out
 */
export function rotate(out: vec2, a: ReadonlyVec2, b: ReadonlyVec2, rad: number): vec2 {
  //Translate point to the origin
  let p0 = a[0] - b[0],
    p1 = a[1] - b[1],
    sinC = Math.sin(rad),
    cosC = Math.cos(rad);

  //perform rotation and translate to correct position
  out[0] = p0 * cosC - p1 * sinC + b[0];
  out[1] = p0 * sinC + p1 * cosC + b[1];

  return out as Vec2;
}

/**
 * Get the smallest angle between two 2D vectors
 * @param {ReadonlyVec2} a The first operand
 * @param {ReadonlyVec2} b The second operand
 * @returns {Number} The angle in radians
 */
export function angle(a: ReadonlyVec2, b: ReadonlyVec2): number {
  let ax = a[0],
    ay = a[1],
    bx = b[0],
    by = b[1];
  return Math.abs(Math.atan2(ay * bx - ax * by, ax * bx + ay * by));
}

/**
 * Get the signed angle in the interval [-pi,pi] between two 2D vectors (positive if `a` is to the right of `b`)
 *
 * @param {ReadonlyVec2} a The first vector
 * @param {ReadonlyVec2} b The second vector
 * @returns {number} The signed angle in radians
 */
export function signedAngle(a: ReadonlyVec2, b: ReadonlyVec2): number {
  let ax = a[0],
    ay = a[1],
    bx = b[0],
    by = b[1];
  return Math.atan2(ax * by - ay * bx, ax * bx + ay * by);
}

/**
 * Set the components of a vec2 to zero
 *
 * @param {vec2} out the receiving vector
 * @returns {vec2} out
 */
export function zero(out: vec2): vec2 {
  out[0] = 0.0;
  out[1] = 0.0;
  return out as Vec2;
}

/**
 * Returns a string representation of a vector
 *
 * @param {ReadonlyVec2} a vector to represent as a string
 * @returns {String} string representation of the vector
 */
export function str(a: ReadonlyVec2): string {
  return 'vec2(' + a[0] + ', ' + a[1] + ')';
}

/**
 * Returns whether or not the vectors exactly have the same elements in the same position (when compared with ===)
 *
 * @param {ReadonlyVec2} a The first vector.
 * @param {ReadonlyVec2} b The second vector.
 * @returns {Boolean} True if the vectors are equal, false otherwise.
 */
export function exactEquals(a: ReadonlyVec2, b: ReadonlyVec2): boolean {
  return a[0] === b[0] && a[1] === b[1];
}

/**
 * Returns whether or not the vectors have approximately the same elements in the same position.
 *
 * @param {ReadonlyVec2} a The first vector.
 * @param {ReadonlyVec2} b The second vector.
 * @returns {Boolean} True if the vectors are equal, false otherwise.
 */
export function equals(a: ReadonlyVec2, b: ReadonlyVec2): boolean {
  let a0 = a[0],
    a1 = a[1];
  let b0 = b[0],
    b1 = b[1];
  return (
    Math.abs(a0 - b0) <= vectorMath.EPSILON * Math.max(1.0, Math.abs(a0), Math.abs(b0)) &&
    Math.abs(a1 - b1) <= vectorMath.EPSILON * Math.max(1.0, Math.abs(a1), Math.abs(b1))
  );
}

/**
 * Alias for {@link vec2.length}
 * @function
 */
export const len = length;

/**
 * Alias for {@link vec2.subtract}
 * @function
 */
export const sub = subtract;

/**
 * Alias for {@link vec2.multiply}
 * @function
 */
export const mul = multiply;

/**
 * Alias for {@link vec2.divide}
 * @function
 */
export const div = divide;

/**
 * Alias for {@link vec2.distance}
 * @function
 */
export const dist = distance;

/**
 * Alias for {@link vec2.squaredDistance}
 * @function
 */
export const sqrDist = squaredDistance;

/**
 * Alias for {@link vec2.squaredLength}
 * @function
 */
export const sqrLen = squaredLength;

/**
 * Perform some operation over an array of vec2s.
 *
 * @param {Array} a the array of vectors to iterate over
 * @param {Number} stride Number of elements between the start of each vec2. If 0 assumes tightly packed
 * @param {Number} offset Number of elements to skip at the beginning of the array
 * @param {Number} count Number of vec2s to iterate over. If 0 iterates over entire array
 * @param {Function} fn Function to call for each vector in the array
 * @param {Object} [arg] additional argument to pass to fn
 * @returns {Array} a
 * @function
 */
export const forEach = (function () {
  let vec = create();

  return function <T>(
    a: number[],
    stride = 2,
    offset = 0,
    count = 0,
    fn: (out: vec2, vector: vec2, arg: T) => vec2,
    arg: T
  ): number[] {
    let i, l;

    if (count) {
      l = Math.min(count * stride + offset, a.length);
    } else {
      l = a.length;
    }

    for (i = offset; i < l; i += stride) {
      vec[0] = a[i] ?? 0;
      vec[1] = a[i + 1] ?? 0;
      fn(vec, vec, arg);
      a[i] = vec[0];
      a[i + 1] = vec[1];
    }

    return a;
  };
})();

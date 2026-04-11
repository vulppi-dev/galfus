export interface IndexedCollection extends Iterable<number> {
  readonly length: number;
  [index: number]: number;
}

export type AngleOrder = 'zyx' | 'xyz' | 'xzy' | 'yxz' | 'yzx' | 'zxy';

export const EPSILON = 0.000001;
export const ARRAY_TYPE = Array;
export let RANDOM: () => number = Math.random;
export let ANGLE_ORDER: AngleOrder = 'zyx';

export function createArray(length: number): number[] {
  return Array.from({ length }, () => 0);
}

/**
 * Symmetric round
 * see https://www.npmjs.com/package/round-half-up-symmetric#user-content-detailed-background
 *
 * @param {Number} a value to round
 */
export function round(a: number): number {
  if (a >= 0) return Math.round(a);

  return a % 0.5 === 0 ? Math.floor(a) : Math.round(a);
}

/**
 * Kept for compatibility with gl-matrix-style APIs.
 * Vulfram math always uses standard JS arrays on the host side.
 *
 * @returns {ArrayConstructor} Array
 */
export function setMatrixArrayType(): ArrayConstructor {
  return ARRAY_TYPE;
}

const degree = Math.PI / 180;

const radian = 180 / Math.PI;

/**
 * Convert Degree To Radian
 *
 * @param {Number} a Angle in Degrees
 */
export function toRadian(a: number): number {
  return a * degree;
}

/**
 * Convert Radian To Degree
 *
 * @param {Number} a Angle in Radians
 */
export function toDegree(a: number): number {
  return a * radian;
}

/**
 * Tests whether or not the arguments have approximately the same value, within an absolute
 * or relative tolerance of vectorMath.EPSILON (an absolute tolerance is used for values less
 * than or equal to 1.0, and a relative tolerance is used for larger values)
 *
 * @param {Number} a          The first number to test.
 * @param {Number} b          The second number to test.
 * @param {Number} tolerance  Absolute or relative tolerance (default vectorMath.EPSILON)
 * @returns {Boolean} True if the numbers are approximately equal, false otherwise.
 */
export function equals(a: number, b: number, tolerance = EPSILON): boolean {
  return Math.abs(a - b) <= tolerance * Math.max(1, Math.abs(a), Math.abs(b));
}

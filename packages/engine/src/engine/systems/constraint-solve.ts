import { mat4 } from 'gl-matrix';
import type { ParentComponent, System, TransformComponent } from '../ecs';
import { mat4EqualsApprox, getEntityLocalTransformMatrix } from './utils';

type ConstraintStrategyContext = {
  worldId: number;
  entityId: number;
};

interface ConstraintStrategy {
  name: string;
  apply(a: mat4, b: mat4, context: ConstraintStrategyContext): mat4;
}

const ParentConstraintStrategy: ConstraintStrategy = {
  name: 'parent',
  apply(a, b) {
    const out = mat4.create();
    mat4.multiply(out, a, b);
    return out;
  }
};

type ResolveState = {
  worldId: number;
  world: Parameters<System>[0];
  affected: Set<number>;
  resolved: Map<number, mat4>;
  visiting: Set<number>;
  cycleLogged: boolean;
};

function resolveEntityMatrix(state: ResolveState, entityId: number): mat4 {
  if (!state.affected.has(entityId)) {
    const existing = state.world.resolvedEntityTransforms.get(entityId);
    if (existing) {
      return existing as unknown as mat4;
    }
  }

  const cached = state.resolved.get(entityId);
  if (cached) {
    return cached;
  }

  if (state.visiting.has(entityId)) {
    if (!state.cycleLogged) {
      console.error(
        `[World ${state.worldId}] Constraint cycle detected in parent hierarchy. Falling back to local transforms for cyclic nodes.`
      );
      state.cycleLogged = true;
    }
    return getEntityLocalTransformMatrix(state.world, entityId);
  }

  state.visiting.add(entityId);
  const local = getEntityLocalTransformMatrix(state.world, entityId);
  const store = state.world.components.get(entityId);
  const parent = store?.get('Parent') as ParentComponent | undefined;

  let resolved = local;
  if (parent) {
    const parentStore = state.world.components.get(parent.parentId);
    const parentTransform = parentStore?.get('Transform') as TransformComponent | undefined;
    if (parentTransform) {
      const parentMatrix = resolveEntityMatrix(state, parent.parentId);
      resolved = ParentConstraintStrategy.apply(parentMatrix, local, {
        worldId: state.worldId,
        entityId
      });
    }
  }

  state.visiting.delete(entityId);
  state.resolved.set(entityId, resolved);
  return resolved;
}

function collectAffectedEntities(world: Parameters<System>[0]): Set<number> {
  const affected = new Set<number>();
  const queue: number[] = [];

  for (const entityId of world.constraintDirtyEntities) {
    affected.add(entityId);
    queue.push(entityId);
  }

  for (let i = 0; i < queue.length; i++) {
    const parentId = queue[i]!;
    const children = world.constraintChildrenByParent.get(parentId);
    if (!children) continue;
    for (const childId of children) {
      if (affected.has(childId)) continue;
      affected.add(childId);
      queue.push(childId);
    }
  }

  return affected;
}

/**
 * Resolves transform constraints into world matrices.
 *
 * Current built-in strategy:
 * - `parent`: composes parent world matrix with child local matrix (`A * B`).
 *
 * The solver walks dependencies from higher nodes to leaves and caches each
 * resolved entity matrix so render sync can emit final transforms to core.
 */
export const ConstraintSolveSystem: System = (world, context) => {
  if (world.constraintDirtyEntities.size === 0) {
    world.constraintChangedEntities.clear();
    return;
  }

  const matrices = world.resolvedEntityTransforms;
  const changed = world.constraintChangedEntities;
  changed.clear();
  const affected = collectAffectedEntities(world);

  const resolveState: ResolveState = {
    worldId: context.worldId,
    world,
    affected,
    resolved: world.constraintScratchResolved,
    visiting: world.constraintScratchVisiting,
    cycleLogged: false
  };
  resolveState.resolved.clear();
  resolveState.visiting.clear();

  for (const entityId of affected) {
    if (!world.entities.has(entityId)) {
      if (matrices.delete(entityId)) {
        changed.add(entityId);
      }
      continue;
    }

    const store = world.components.get(entityId);
    const transform = store?.get('Transform') as TransformComponent | undefined;
    if (!transform) {
      if (matrices.delete(entityId)) {
        changed.add(entityId);
      }
      continue;
    }

    const resolved = resolveEntityMatrix(resolveState, entityId);
    const previousSnapshot = matrices.get(entityId);
    if (!previousSnapshot) {
      matrices.set(entityId, new Float32Array(resolved));
      changed.add(entityId);
      continue;
    }

    if (!mat4EqualsApprox(previousSnapshot, resolved)) {
      previousSnapshot.set(resolved);
      changed.add(entityId);
    }
  }

  world.constraintDirtyEntities.clear();
};

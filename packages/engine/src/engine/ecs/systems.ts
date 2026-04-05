import type { WorldState } from '../state';
import type { ComponentSchema } from './components';

/** Pipeline stage at which a system runs. */
export type SystemStep = 'input' | 'update' | 'preRender' | 'postRender';

/** System execution context provided each frame. */
export interface SystemContext {
  dt: number;
  time: number;
  worldId: number;
}

/** System function signature. */
export type System = (state: WorldState, context: SystemContext) => void;

/** Registry of components and systems. */
export interface EngineRegistry {
  components: Map<string, ComponentSchema>;
  systems: {
    input: System[];
    update: System[];
    preRender: System[];
    postRender: System[];
  };
}

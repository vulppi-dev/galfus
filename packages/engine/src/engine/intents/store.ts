import type { Intent } from '../ecs';

export type IntentType = Intent['type'];

type IntentEntry<K extends IntentType> = {
  seq: number;
  intent: Extract<Intent, { type: K }>;
};

export class IntentStore {
  private byType = new Map<IntentType, IntentEntry<IntentType>[]>();
  private sequence = 0;
  private total = 0;

  enqueue(intent: Intent): void {
    const key = intent.type as IntentType;
    const list = this.byType.get(key);
    const entry: IntentEntry<IntentType> = {
      seq: this.sequence++,
      intent: intent as Extract<Intent, { type: IntentType }>,
    };
    if (list) {
      list.push(entry);
    } else {
      this.byType.set(key, [entry]);
    }
    this.total++;
  }

  take<K extends IntentType>(type: K): Extract<Intent, { type: K }>[] {
    const list = this.byType.get(type);
    if (!list || list.length === 0) {
      return [];
    }
    this.byType.delete(type);
    this.total -= list.length;
    return list.map((entry) => entry.intent as Extract<Intent, { type: K }>);
  }

  takeMany<K extends IntentType>(
    types: readonly K[],
  ): Extract<Intent, { type: K }>[] {
    const merged: IntentEntry<K>[] = [];
    for (let i = 0; i < types.length; i++) {
      const type = types[i];
      if (type === undefined) continue;
      const list = this.byType.get(type);
      if (!list || list.length === 0) continue;
      this.byType.delete(type);
      this.total -= list.length;
      for (let j = 0; j < list.length; j++) {
        const entry = list[j];
        if (entry) merged.push(entry as unknown as IntentEntry<K>);
      }
    }
    merged.sort((a, b) => a.seq - b.seq);
    return merged.map((entry) => entry.intent as Extract<Intent, { type: K }>);
  }

  size(): number {
    return this.total;
  }

  clear(): void {
    this.byType.clear();
    this.total = 0;
  }
}

export function createIntentStore(): IntentStore {
  return new IntentStore();
}

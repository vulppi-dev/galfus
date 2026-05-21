export const DEMO_IDS = ['001'] as const;

export type DemoId = (typeof DEMO_IDS)[number];

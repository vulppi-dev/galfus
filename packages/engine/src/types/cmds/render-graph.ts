export type LogicalId = string | number;

export type RenderGraphResourceKind = 'texture' | 'buffer' | 'attachment';

export type RenderGraphLifetime = 'frame' | 'persistent';

export type RenderGraphEdgeReason = 'read-after-write' | 'write-after-read';

export type RenderGraphValue = boolean | number | string;

export interface RenderGraphResource {
  resId: LogicalId;
  kind?: RenderGraphResourceKind;
  lifetime?: RenderGraphLifetime;
  aliasGroup?: LogicalId | null;
}

export interface RenderGraphNode {
  nodeId: LogicalId;
  passId: string;
  inputs?: LogicalId[];
  outputs?: LogicalId[];
  params?: Record<string, RenderGraphValue>;
}

export interface RenderGraphEdge {
  fromNodeId: LogicalId;
  toNodeId: LogicalId;
  reason?: RenderGraphEdgeReason;
}

export interface RenderGraphDesc {
  graphId: LogicalId;
  nodes: RenderGraphNode[];
  edges: RenderGraphEdge[];
  resources?: RenderGraphResource[];
  fallback?: boolean;
}

export interface CmdRenderGraphUpsertArgs {
  renderGraphId: number;
  graph: RenderGraphDesc;
}

export interface CmdResultRenderGraphUpsert {
  success: boolean;
  message: string;
}

export interface CmdRenderGraphDisposeArgs {
  renderGraphId: number;
}

export interface CmdResultRenderGraphDispose {
  success: boolean;
  message: string;
}

export interface CmdRenderGraphListArgs {}

export interface RenderGraphEntry {
  renderGraphId: number;
  descHash: number;
  passCount: number;
  passIds: string[];
  boundRealmIds: number[];
}

export interface CmdResultRenderGraphList {
  success: boolean;
  message: string;
  renderGraphs: RenderGraphEntry[];
}

export interface CmdRealmRenderGraphBindArgs {
  realmId: number;
  renderGraphId: number;
}

export interface CmdResultRealmRenderGraphBind {
  success: boolean;
  message: string;
}

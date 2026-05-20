export interface ResourceEntry {
  id: number;
  label: string | null;
}

export interface QueryScopeArgs {
  windowId?: number;
  realmId?: number;
  ids?: number[];
}

export interface CmdResourceGetArgs {
  id: number;
  scope?: QueryScopeArgs;
}

export interface CmdResultResourceGet {
  success: boolean;
  message: string;
  kind: string;
  id?: number;
  label?: string | null;
  realmId?: number;
}

export interface CmdResourceListArgs {
  scope?: QueryScopeArgs;
}

export interface CmdResultResourceList {
  success: boolean;
  message: string;
  kind: string;
  items: ResourceEntry[];
}

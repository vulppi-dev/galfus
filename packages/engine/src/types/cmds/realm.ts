export type RealmKind = 'three-d' | 'two-d';

export interface CmdRealmCreateArgs {
  kind: RealmKind;
  importance?: number;
  cachePolicy?: number;
  flags?: number;
}

export interface CmdResultRealmCreate {
  success: boolean;
  message: string;
  realmId?: number;
}

export interface CmdRealmDisposeArgs {
  realmId: number;
}

export interface CmdResultRealmDispose {
  success: boolean;
  message: string;
}

export interface CmdRealmGetArgs {
  realmId: number;
}

export interface CmdResultRealmGet {
  success: boolean;
  message: string;
  realmId: number;
  kind?: RealmKind;
  renderGraphId?: number;
}

export interface CmdRealmListArgs {
  kind?: RealmKind;
  ids?: number[];
}

export interface RealmListItem {
  realmId: number;
  kind: RealmKind;
  renderGraphId?: number;
}

export interface CmdResultRealmList {
  success: boolean;
  message: string;
  items: RealmListItem[];
}

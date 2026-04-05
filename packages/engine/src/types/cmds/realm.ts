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

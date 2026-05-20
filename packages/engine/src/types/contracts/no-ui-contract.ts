import type { EngineCmd, CommandResponse } from '../cmds';
import type { EngineEvent } from '../events';
import type { SystemEvent } from '../events/system';

type AssertNever<T extends never> = T;

type UiEngineCmd = Extract<EngineCmd, { type: `cmd-ui-${string}` }>;
type UiCommandResponse = Extract<CommandResponse, { type: `ui-${string}` }>;
type UiEngineEvent = Extract<EngineEvent, { type: 'ui' }>;
type UiSystemEvent = Extract<SystemEvent, { event: `ui-${string}` }>;

type _assertNoUiEngineCmd = AssertNever<UiEngineCmd>;
type _assertNoUiCommandResponse = AssertNever<UiCommandResponse>;
type _assertNoUiEngineEvent = AssertNever<UiEngineEvent>;
type _assertNoUiSystemEvent = AssertNever<UiSystemEvent>;


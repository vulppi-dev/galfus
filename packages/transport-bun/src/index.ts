import type { EngineTransportFactory } from '@vulfram/transport-types';
import { VULFRAM_CORE } from './bind/ffi-loader';

export const transportBunFfi: EngineTransportFactory = () => VULFRAM_CORE;

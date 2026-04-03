import type { EngineTransportFactory } from '@vulfram/transport-types';
import { VULFRAM_CORE } from './bind/napi-loader';

export const transportNapi: EngineTransportFactory = () => VULFRAM_CORE;

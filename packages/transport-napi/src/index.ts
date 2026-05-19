import type { EngineTransportFactory } from '@galfus/transport-types';
import { GALFUS_CORE } from './bind/napi-loader';

export const transportNapi: EngineTransportFactory = () => GALFUS_CORE;

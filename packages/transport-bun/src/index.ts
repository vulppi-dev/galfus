import type { EngineTransportFactory } from '@galfus/transport-types';
import { GALFUS_CORE } from './bind/ffi-loader';

export const transportBunFfi: EngineTransportFactory = () => GALFUS_CORE;

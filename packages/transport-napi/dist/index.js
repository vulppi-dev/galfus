var __create = Object.create;
var __getProtoOf = Object.getPrototypeOf;
var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
function __accessProp(key) {
  return this[key];
}
var __toESMCache_node;
var __toESMCache_esm;
var __toESM = (mod, isNodeMode, target) => {
  var canCache = mod != null && typeof mod === "object";
  if (canCache) {
    var cache = isNodeMode ? __toESMCache_node ??= new WeakMap : __toESMCache_esm ??= new WeakMap;
    var cached = cache.get(mod);
    if (cached)
      return cached;
  }
  target = mod != null ? __create(__getProtoOf(mod)) : {};
  const to = isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target;
  for (let key of __getOwnPropNames(mod))
    if (!__hasOwnProp.call(to, key))
      __defProp(to, key, {
        get: __accessProp.bind(mod, key),
        enumerable: true
      });
  if (canCache)
    cache.set(mod, to);
  return to;
};
var __commonJS = (cb, mod) => () => (mod || cb((mod = { exports: {} }).exports, mod), mod.exports);

// lib/macos-arm64/vulfram_core.node
var require_vulfram_core = __commonJS((exports, module) => {
  module.exports = "./vulfram_core-sa2ckr6h.node";
});

// lib/macos-x64/vulfram_core.node
var require_vulfram_core2 = __commonJS((exports, module) => {
  module.exports = "./vulfram_core-9zn53dg5.node";
});

// lib/linux-arm64/vulfram_core.node
var require_vulfram_core3 = __commonJS((exports, module) => {
  module.exports = "./vulfram_core-41yvp642.node";
});

// lib/linux-x64/vulfram_core.node
var require_vulfram_core4 = __commonJS((exports, module) => {
  module.exports = "./vulfram_core-vhndqjh5.node";
});

// lib/windows-arm64/vulfram_core.node
var require_vulfram_core5 = __commonJS((exports, module) => {
  module.exports = "./vulfram_core-vgn81mvs.node";
});

// lib/windows-x64/vulfram_core.node
var require_vulfram_core6 = __commonJS((exports, module) => {
  module.exports = "./vulfram_core-wbr6x7wr.node";
});

// ../transport-types/src/index.ts
function detectRuntime() {
  if (typeof globalThis.Deno !== "undefined" && typeof globalThis.Deno?.version?.deno === "string") {
    const deno = globalThis.Deno;
    return {
      runtime: "deno",
      version: deno.version.deno,
      platform: deno.build?.os ?? null,
      arch: deno.build?.arch ?? null
    };
  }
  if (typeof globalThis.Bun !== "undefined" && typeof globalThis.Bun?.version === "string") {
    return {
      runtime: "bun",
      version: globalThis.Bun.version,
      platform: typeof process !== "undefined" ? process.platform : null,
      arch: typeof process !== "undefined" ? process.arch : null
    };
  }
  if (typeof globalThis.process !== "undefined" && typeof globalThis.process?.versions?.node === "string") {
    const proc = globalThis.process;
    return {
      runtime: "node",
      version: proc.versions.node,
      platform: proc.platform ?? null,
      arch: proc.arch ?? null
    };
  }
  return {
    runtime: "unknown",
    version: null,
    platform: null,
    arch: null
  };
}
function selectPlatformLoader(loaders, artifactKind) {
  const runtime = detectRuntime();
  const platformKey = runtime.platform ?? "";
  const archKey = runtime.arch ?? "";
  const byPlatform = loaders[platformKey];
  const selected = byPlatform?.[archKey];
  if (selected)
    return selected;
  throw new Error(`${artifactKind} build not found for the current runtime: ${JSON.stringify(runtime)}`);
}
function resolveNativePlatform(runtime = detectRuntime()) {
  const platform = runtime.platform;
  const arch = runtime.arch;
  if (platform === "linux" && arch === "x64")
    return "linux-x64";
  if (platform === "linux" && arch === "arm64")
    return "linux-arm64";
  if (platform === "darwin" && arch === "x64")
    return "macos-x64";
  if (platform === "darwin" && arch === "arm64")
    return "macos-arm64";
  if (platform === "win32" && arch === "x64")
    return "windows-x64";
  if (platform === "win32" && arch === "arm64")
    return "windows-arm64";
  throw new Error(`Unsupported native platform for Vulfram transports: ${JSON.stringify(runtime)}`);
}
function getArtifactFileName(binding, platform) {
  if (binding === "napi")
    return "vulfram_core.node";
  if (binding === "wasm")
    return "vulfram_core_bg.wasm";
  if (binding === "ffi" && platform.startsWith("windows"))
    return "vulfram_core.dll";
  if (binding === "ffi" && platform.startsWith("macos"))
    return "vulfram_core.dylib";
  if (binding === "ffi")
    return "vulfram_core.so";
  if (platform.startsWith("windows"))
    return "vulfram_core.dll";
  if (platform.startsWith("macos"))
    return "vulfram_core.dylib";
  return "vulfram_core.so";
}

// src/bind/napi-loader.ts
import { createRequire } from "module";
var requireNative = createRequire(import.meta.url);
var loaders = {
  darwin: {
    arm64: () => Promise.resolve().then(() => __toESM(require_vulfram_core(), 1)),
    x64: () => Promise.resolve().then(() => __toESM(require_vulfram_core2(), 1))
  },
  linux: {
    arm64: () => Promise.resolve().then(() => __toESM(require_vulfram_core3(), 1)),
    x64: () => Promise.resolve().then(() => __toESM(require_vulfram_core4(), 1))
  },
  win32: {
    arm64: () => Promise.resolve().then(() => __toESM(require_vulfram_core5(), 1)),
    x64: () => Promise.resolve().then(() => __toESM(require_vulfram_core6(), 1))
  }
};
function getExpectedLocalArtifact() {
  try {
    const platform = resolveNativePlatform();
    const filename = getArtifactFileName("napi", platform);
    return `../../lib/${platform}/${filename}`;
  } catch {
    return "../../lib/<platform>/vulfram_core.node";
  }
}
async function resolveNativeModulePath() {
  const importLoader = selectPlatformLoader(loaders, "N-API");
  try {
    return (await importLoader()).default;
  } catch (error) {
    const runtime = detectRuntime();
    const expectedArtifact = getExpectedLocalArtifact();
    throw new Error(`Failed to load bundled N-API artifact (runtime=${runtime.runtime}, platform=${runtime.platform ?? "unknown"}, arch=${runtime.arch ?? "unknown"}, expected=${expectedArtifact}): ${String(error)}`);
  }
}
var modulePath = await resolveNativeModulePath();
var raw = requireNative(modulePath);
var VULFRAM_CORE = {
  vulframInit: () => raw.vulframInit(),
  vulframDispose: () => raw.vulframDispose(),
  vulframReceiveQueue: () => raw.vulframReceiveQueue(),
  vulframReceiveEvents: () => raw.vulframReceiveEvents(),
  vulframTick: (timeMs, deltaMs) => raw.vulframTick(timeMs, deltaMs),
  vulframGetProfiling: () => raw.vulframGetProfiling(),
  vulframSendQueue: (buffer) => {
    const data = Buffer.isBuffer(buffer) ? buffer : Buffer.from(buffer);
    return raw.vulframSendQueue(data);
  },
  vulframUploadBuffer: (id, uploadType, buffer) => {
    const data = Buffer.isBuffer(buffer) ? buffer : Buffer.from(buffer);
    return raw.vulframUploadBuffer(id, uploadType, data);
  }
};

// src/index.ts
var transportNapi = () => VULFRAM_CORE;
export {
  transportNapi
};

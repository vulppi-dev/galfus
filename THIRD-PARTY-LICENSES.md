# Third-Party Licenses

This file contains a list of third-party libraries used in **Vulfram**, along with their respective licenses.

## Summary

| License                                | Count |
| :------------------------------------- | :---- |
| Apache-2.0 / MIT (Dual)                | 268   |
| MIT                                    | 101   |
| Apache-2.0                             | 10    |
| BSD (2/3-Clause)                       | 10    |
| Zlib / ISC                             | 13    |
| MPL-2.0                                | 11    |
| Other Permissive (Unlicense, CC0, ISC) | 15    |

---

## Detailed List by License

### Apache-2.0 OR MIT

_Used by many foundational crates including wgpu, winit, and serde._

- ahash, aligned, allocator-api2, android-activity, anstream, anstyle, anyhow, arrayvec, ash, async-broadcast, async-channel, async-executor, async-io, async-lock, async-process, async-recursion, async-signal, async-task, async-trait, atomic-waker, autocfg, bit-set, bit-vec, bitflags, blocking, bumpalo, bytemuck, cc, cfg-if, colorchoice, concurrent-queue, core-foundation, core2, crc32fast, crossbeam-deque, crossbeam-epoch, crossbeam-utils, ctrlc, ctor, deranged, document-features, dpi, dtor, egui, either, enumflags2, env_filter, env_logger, equator, equivalent, errno, event-listener, fastrand, fdeflate, flate2, futures, getrandom, gilrs, glam, glow, gpu-allocator, half, hashbrown, hermit-abi, hex, image, indexmap, is_terminal_polyfill, itertools, jni, jobserver, js-sys, kira, libc, litrs, lock_api, log, memmap2, metal, miniz_oxide, naga, ndk, nohash-hasher, notify-rust, num-bigint, num-derive, num-integer, num-traits, once_cell, parking, parking_lot, paste, percent-encoding, pin-project, pin-utils, pkg-config, png, pollster, portable-atomic, powerfmt, presser, proc-macro-crate, proc-macro2, profiling, pyo3, quote, rand, range-alloc, raw-window-handle, rayon, regex, renderdoc-sys, rustc-hash, rustversion, scopeguard, semver, serde, serde_repr, shlex, smallvec, smol_str, static_assertions, syn, tempfile, thiserror, time, toml_edit, ttf-parser, unicode-segmentation, unicode-width, utf8parse, uuid, wasm-bindgen, wasm-bindgen-futures, web-sys, wgpu, windows, x11rb, zune-core, and others.

### MIT

- aligned-vec, android-properties, av-scenechange, block, built, bytes, calloop, cfg_aliases, combine, convert_case, crunchly, dispatch, dlib, equator-macro, gl-matrix (vendored in `packages/engine/src/math`), interpolate_name, libm, libredox, libudev-sys, loop9, malloc_buf, maybe-rayon, memoffset, mint, mlua, napi, napi-build, napi-derive, napi-sys, new_debug_unreachable, nix, nom, noop_proc_macro, objc, objc2, orbclient, ordered-float, quick-xml, redox_syscall, rgb, rmp, rmp-serde, sctk-adwaita, simd-adler32, slab, smithay-client-toolkit, strict-num, tracing, uds_windows, wayland-backend, wayland-client, wayland-protocols, wayland-scanner, wayland-sys, winnow, x11-dl, xcursor, xkbcommon-dl, xml-rs, y4m, zbus, zvariant, and others.

### MPL-2.0

- symphonia, symphonia-bundle-flac, symphonia-bundle-mp3, symphonia-codec-pcm, symphonia-codec-vorbis, symphonia-core, symphonia-format-ogg, symphonia-format-riff, symphonia-metadata, symphonia-utils-xiph, triple_buffer.

### Apache-2.0

- ab_glyph, ab_glyph_rasterizer, codespan-reporting, gethostname, gl_generator, glutin_wgl_sys, khronos_api, owned_ttf_parser, spirv, winit.

### BSD-2-Clause / BSD-3-Clause

- arrayref, av1-grain, avif-serialize, rav1e, ravif, tiny-skia, tiny-skia-path, v_frame.

### Zlib

- bytemuck, bytemuck_derive, const_panic, dispatch2, foldhash, slotmap, typewit.

### ISC

- inotify, inotify-sys, libloading.

### Unlicense OR MIT

- aho-corasick, byteorder-lite, jiff, memchr, same-file, termcolor, walkdir, winapi-util.

### MIT-0

- encase, encase_derive, encase_derive_impl.

---

_This file was automatically generated based on the project's dependency tree._

# AGENTS.md — Vulfram instructions

- Never directly confirm a proposal in a positive way; you must logically assess whether it is actually valid or not. If it is not, suggest a proposal that is better aligned with solving the problem effectively and efficiently.
- Planning must be done without modifying files (without generating code) when the user asks only for analysis.
- Variables that hold ownership and are no longer used afterward must always receive the `_` prefix.
- If variables are unused, they must be removed.
- Unused functions must also be removed.
- Files should target around 300 lines and at most 600 lines. If they exceed that and it is possible, split them into smaller files.
- Always update the related documentation when finishing a phase.
- In future audits, ignore retention caused by host-side resources without dispose.
- Internal Rust properties use `snake_case`; serde converts them to `camelCase` on the host side.
- Resources must follow this pattern: logical IDs are controlled by the host, while all other management and control requirements remain in the core, including physical ID generation, pooling, etc.
- The host is responsible for ensuring that logical IDs are unique and valid; the core assumes this is true and performs no extra validation.

# Native plugins (C / Zig / …)

v2 treats **native shared libraries** as a plugin tier on top of the Rust `PluginHost`. The recommended path for generating safe FFI glue is **[equilibrium](https://github.com/semitechnological/equilibrium)** (automatic C FFI generation for C-compiling languages).

## Layout

- Optional git submodule: `vendor/equilibrium` (run `git submodule update --init vendor/equilibrium` after clone).
- Your native code produces a `.so` / `.dylib` / `.dll` with a C ABI exported through equilibrium-generated headers.

## Registration (stub)

The in-tree `PluginHost` currently registers **Rust tools** loaded at startup. Dynamic `dlopen` of third-party `.so` files, ABI version negotiation, and signing policy are **not** enabled in the default build; they are documented here so native plugins can follow a stable integration path as the loader matures.

## Security

- Treat native plugins as **fully trusted** code with the same privileges as the unthinkclaw process.
- Prefer loading only from user-owned paths under `.unthinkclaw/plugins/` and verify hashes out-of-band before first load.

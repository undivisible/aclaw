# Plugin packages (manifest)

Named packages in `.unthinkclaw/plugins/manifest.json` expand to toolset groups via `tools::toolsets::expand_package` and `apply_package_manifest`.

| Package id        | Toolset groups enabled                          |
|-------------------|-------------------------------------------------|
| `core` / `default`| runtime, fs, memory, sessions, misc             |
| `web`             | web                                             |
| `browser`         | browser                                         |
| `skills`          | skills                                          |
| `advanced`        | advanced                                        |
| `unthinkclaw-live`| web, browser, skills, advanced                  |

Core groups are always merged first when any package list is non-empty so shell/files/memory stay available unless explicitly disabled via `toolsets.disabled` in the manifest.

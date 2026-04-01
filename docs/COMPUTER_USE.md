# Computer Use Implementation

## Overview

This document describes the computer use capabilities added to unthinkclaw and the integration with poke-around.

## Architecture

### unthinkclaw (Rust)

Computer use is implemented as a native tool following unthinkclaw's trait-based architecture:

```
src/tools/computer_use.rs               # Main tool implementation
src/tools/computer_use/platforms/
  ├── mod.rs                             # Platform abstraction trait
  ├── fallback.rs                        # Basic input simulation (enigo)
  ├── linux.rs                           # Linux (AT-SPI2 + X11/Wayland)
  ├── macos.rs                           # macOS (Accessibility + CoreGraphics)
  └── windows.rs                         # Windows (UI Automation + Win32)
```

**Tool Operations:**
- `computer.click(target, button?)` - Click element by coordinates/ID/path
- `computer.type(text, target?)` - Type text at cursor or target element
- `computer.screenshot(area?)` - Capture full screen or region
- `computer.scroll(direction, amount)` - Scroll view
- `computer.key(keys)` - Send keyboard shortcuts (e.g., "Ctrl+C")
- `computer.mouse_move(x, y)` - Move mouse cursor
- `computer.inspect(target?)` - Get accessibility tree or element info

**Cargo Features:**
- `computer-use` - Base feature (enigo + image)
- `computer-use-linux` - Linux-specific (xcb, ashpd)
- `computer-use-macos` - macOS-specific (accessibility, core-graphics)
- `computer-use-windows` - Windows-specific (windows crate)
- `computer-use-ocr` - Optional OCR fallback (tesseract)

### poke-around (Zig)

poke-around is a standalone MCP server that exposes tools to Poke AI. It's ready to use and has:

**Current Tools (13):**
1. `run_command` - Execute shell commands
2. `network_speed` - Internet speed test
3. `read_file` - Read file contents
4. `write_file` - Write to files
5. `list_directory` - List directory contents
6. `system_info` - Get system information
7. `read_image` - Read binary/image files as base64
8. `run_agent` - Run Poke Around agents
9. `take_screenshot` - Capture screen
10. `edit_file` - Surgical file editing
11. `web_fetch` - Fetch URLs
12. `http_request` - Custom HTTP requests
13. `git_operations` - Git operations

**Planned Computer Use Tools:**
- `computer_click` - Click at coordinates
- `computer_type` - Type text
- `computer_key` - Keyboard shortcuts
- `computer_mouse_move` - Move mouse
- `computer_scroll` - Scroll view

## Implementation Status

### Phase 1: Computer Use Core (unthinkclaw)
- ✅ Tool trait with all operations
- ✅ Platform abstraction layer  
- ✅ Fallback implementation (enigo-based)
- ✅ Linux platform stub (AT-SPI2 ready)
- ✅ macOS platform stub (Accessibility ready)
- ✅ Windows platform stub (UI Automation ready)
- ⏳ Self-correction loop integration (pending)

### Phase 2: Poke-Around Bridge
- ✅ unthinkclaw MCP server exposes computer use tool
- ⏳ poke-around native computer use tools (Zig implementation)
- ⏳ Optional bridge to unthinkclaw's MCP server

### Phase 3: Swarm Integration
- ⏳ Capability registration in SurrealDB
- ⏳ Workload routing across machines
- ⏳ Unified Poke identity for swarm

### Phase 4: Enhanced Interaction Loop
- ⏳ State observation in ToolResult
- ⏳ Visual feedback in agent loop

## Usage

### unthinkclaw

Build with computer use support:
```bash
cd /home/undivisible/unthinkclaw
cargo build --release --features computer-use

# Platform-specific builds
cargo build --release --features computer-use-linux    # Linux
cargo build --release --features computer-use-macos    # macOS  
cargo build --release --features computer-use-windows  # Windows
```

The computer use tool is automatically available in MCP mode:
```bash
./unthinkclaw --mcp-http 8080
```

### poke-around

poke-around is already built and ready:
```bash
cd /home/undivisible/poking-around
./zig-out/bin/poke-around
```

Access modes:
- `--mode full` - All tools, approval required for risky operations
- `--mode limited` - Read-only + safe commands
- `--mode sandbox` - Broader commands, restricted write paths

## Security

### unthinkclaw
- Deny-by-default for channel allowlists
- Workspace-scoped filesystem access
- No logging of tokens, keys, or message content
- Input validation at tool boundaries

### poke-around
- Three access modes with graduated permissions
- Approval tokens for risky operations
- Session-based approval memory
- Command allowlists for limited/sandbox modes

## Integration Pattern

### Option 1: Standalone poke-around
Use poke-around's native tools directly (current state):
```
Poke Agent → PokeTunnel (WebSocket) → poke-around MCP server → Native Zig tools
```

### Option 2: Hybrid (future)
poke-around can optionally proxy to unthinkclaw for advanced features:
```
Poke Agent → PokeTunnel → poke-around → HTTP → unthinkclaw MCP → Rust tools
```

### Option 3: Direct unthinkclaw
For non-Poke use cases, connect directly to unthinkclaw:
```
MCP Client → HTTP → unthinkclaw MCP server → Rust tools
```

## Next Steps

1. **Complete platform implementations**
   - Implement AT-SPI2 for Linux accessibility
   - Implement AXUIElement for macOS accessibility
   - Implement UI Automation for Windows
   - Add real screenshot capture (currently placeholders)

2. **Add computer use to poke-around**
   - Implement native Zig versions of computer use tools
   - Add to TOOLS_JSON and tool dispatch
   - Test with Poke Agent

3. **Self-correction loop**
   - Capture state after each action
   - Return visual/structural observations
   - Enable agent validation and retry

4. **Swarm coordination**
   - Register capabilities in SurrealDB
   - Implement task routing
   - Test multi-machine scenarios

## References

- [unthinkclaw AGENTS.md](../AGENTS.md) - Engineering protocol
- [poke-around README](../../poking-around/README.md) - User documentation
- [MCP Specification](https://modelcontextprotocol.io) - Protocol details
- [Poke SDK](https://www.npmjs.com/package/poke) - Tunnel implementation

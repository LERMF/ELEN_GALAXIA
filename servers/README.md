# 🏗️ NEXUS Server Farm

This directory is reserved for custom or locally-built MCP servers that are not available via `npm` or `docker`.

## Recommended Usage

1. **Clone/Create:** Place the source code of the MCP server here.
2. **Build:** Ensure `npm install && npm run build` (or equivalent) is successful.
3. **Register:** Update `global-config/mcp_registry.json` to point to the local path:

```json
"my-local-server": {
  "command": "node",
  "args": ["/absolute/path/to/nexus-protocol/servers/my-local-server/build/index.js"]
}
```

## Structure

- `/servers`: Source code for custom servers.
- `/global-config/mcp_registry.json`: The "switchboard" configuration.

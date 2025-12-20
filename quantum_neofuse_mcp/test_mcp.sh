#!/bin/bash
# Test Initialize
echo "--- Testing Initialize ---"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/quantum_neofuse_mcp

# Test Tools List
echo -e "\n--- Testing Tools List ---"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | ./target/release/quantum_neofuse_mcp

# Test Quantum Sim Tool
echo -e "\n--- Testing Quantum Sim Tool ---"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"quantum_sim","arguments":{"branches":3}}}' | ./target/release/quantum_neofuse_mcp

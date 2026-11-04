#!/bin/sh
# Pipes a JSON-RPC session into the server. Usage: S5_MODULES=notes,plan sh run.sh
BIN=${BIN:-target/debug/s5-mcp}
{
  echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"pipe","version":"0"}}}'
  echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
  echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"notes.add","arguments":{"text":"buy milk"}}}'; sleep ${GAP:-0}
  echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"notes.add","arguments":{"text":"  "}}}'; sleep ${GAP:-0}
  echo '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"notes.list","arguments":{}}}'; sleep ${GAP:-0}
  echo '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"plan.today","arguments":{}}}'; sleep ${GAP:-0}
  sleep 1
} | "$BIN"

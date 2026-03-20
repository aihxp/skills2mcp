#!/usr/bin/env bash
# Wall-clock benchmarks for sxmc (median of N runs). Uses repo tests/fixtures only.
# Usage: scripts/benchmark_cli.sh [output.md]
# Env: SXMC=path/to/sxmc (default: sxmc on PATH)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FIXTURES="$ROOT/tests/fixtures"
SXMC="${SXMC:-sxmc}"
OUT="${1:-$ROOT/benchmark-results.md}"
RUNS="${RUNS:-5}"

median_ms() {
  local -a arr=("$@")
  local n=${#arr[@]}
  IFS=$'\n' sorted=($(printf '%s\n' "${arr[@]}" | sort -n))
  local mid=$((n / 2))
  echo "${sorted[$mid]}"
}

time_ms() {
  local t0 t1
  t0=$(date +%s%3N)
  "$@" >/dev/null 2>&1 || return 1
  t1=$(date +%s%3N)
  echo $((t1 - t0))
}

time_ms_any() {
  local t0 t1
  t0=$(date +%s%3N)
  "$@" >/dev/null 2>&1 || true
  t1=$(date +%s%3N)
  echo $((t1 - t0))
}

report_line() {
  printf '%s\n' "$*" >>"$OUT"
}

: >"$OUT"
report_line "# sxmc benchmark run — automated"
report_line ""
report_line "- **When:** $(date -Iseconds)"
report_line "- **sxmc:** $($SXMC --version 2>&1)"
report_line "- **Repo fixtures:** \`$FIXTURES\`"
report_line "- **Runs per timing:** ${RUNS} (median ms)"
report_line ""

# A — stdio → fixture script tool
report_line "## A — stdio bridge → skill script (fixtures)"
declare -a a_times=()
for ((i=0; i<RUNS; i++)); do
  ms=$(time_ms "$SXMC" stdio "$SXMC serve --paths $FIXTURES" skill_with_scripts__hello) || ms=999999
  a_times+=("$ms")
done
report_line "| Metric | Value |"
report_line "|--------|-------|"
report_line "| stdio serve --paths tests/fixtures → skill_with_scripts__hello (median ms) | $(median_ms "${a_times[@]}") |"
report_line ""
report_line 'Sample output:'
report_line '```'
"$SXMC" stdio "$SXMC serve --paths $FIXTURES" skill_with_scripts__hello 2>&1 | head -8 >>"$OUT"
report_line '```'
report_line ""

# B — Petstore
report_line "## B — OpenAPI Petstore"
declare -a b_sxmc_list=() b_sxmc_call=() b_curl=()
SPEC_URL="https://petstore3.swagger.io/api/v3/openapi.json"
API_URL="https://petstore3.swagger.io/api/v3/pet/findByStatus?status=available"
for ((i=0; i<RUNS; i++)); do
  ms=$(time_ms "$SXMC" api "$SPEC_URL" --list) || ms=999999
  b_sxmc_list+=("$ms")
done
for ((i=0; i<RUNS; i++)); do
  ms=$(time_ms "$SXMC" api "$SPEC_URL" findPetsByStatus status=available) || ms=999999
  b_sxmc_call+=("$ms")
done
for ((i=0; i<RUNS; i++)); do
  ms=$(time_ms curl -sS "$API_URL" -o /dev/null) || ms=999999
  b_curl+=("$ms")
done
report_line "### sxmc api"
report_line "| Step | Median ms |"
report_line "|------|-----------|"
report_line "| api --list | $(median_ms "${b_sxmc_list[@]}") |"
report_line "| api findPetsByStatus | $(median_ms "${b_sxmc_call[@]}") |"
report_line ""
report_line "### curl only (lower bound)"
report_line "| Step | Median ms |"
report_line "|------|-----------|"
report_line "| curl GET findByStatus | $(median_ms "${b_curl[@]}") |"
report_line ""

# C — nested list
report_line "## C — stdio nested serve --list"
declare -a c_times=()
for ((i=0; i<RUNS; i++)); do
  ms=$(time_ms "$SXMC" stdio "$SXMC serve --paths $FIXTURES" --list) || ms=999999
  c_times+=("$ms")
done
report_line "| stdio nested --list (median ms) | $(median_ms "${c_times[@]}") |"
report_line ""

# D — scan
report_line "## D — scan malicious-skill"
declare -a d_times=()
for ((i=0; i<RUNS; i++)); do
  ms=$(time_ms_any "$SXMC" scan --paths "$FIXTURES" --skill malicious-skill)
  d_times+=("$ms")
done
report_line "| scan --skill malicious-skill (median ms) | $(median_ms "${d_times[@]}") |"
report_line ""

# Micro (ephemeral port via env)
PORT=$(( 8800 + RANDOM % 800 ))
export SXMC_BENCH_PORT="$PORT"
MINI_SPEC="/tmp/sxmc-mini-openapi-$$.json"
cat >"$MINI_SPEC" <<JSON
{
  "openapi": "3.0.0",
  "info": { "title": "Mini", "version": "1.0" },
  "servers": [{ "url": "http://127.0.0.1:${PORT}" }],
  "paths": {
    "/pets": {
      "get": {
        "operationId": "listPets",
        "parameters": [
          { "name": "status", "in": "query", "schema": { "type": "string" } }
        ],
        "responses": { "200": { "description": "ok" } }
      }
    }
  }
}
JSON

python3 <<'PY' &
import json
import os
from http.server import HTTPServer, BaseHTTPRequestHandler

port = int(os.environ["SXMC_BENCH_PORT"])

class H(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path.startswith("/pets"):
            b = json.dumps([{"id": 1, "status": "available"}]).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b)
        else:
            self.send_response(404)
            self.end_headers()
    def log_message(self, *a):
        pass

HTTPServer(("127.0.0.1", port), H).serve_forever()
PY
MINI_PID=$!
sleep 0.35
declare -a m_times=()
for ((i=0; i<RUNS; i++)); do
  ms=$(time_ms_any "$SXMC" api "$MINI_SPEC" listPets status=available)
  m_times+=("$ms")
done
kill $MINI_PID 2>/dev/null || true
wait $MINI_PID 2>/dev/null || true
rm -f "$MINI_SPEC"

report_line "## Micro — local OpenAPI + tiny HTTP server"
report_line "| api listPets (median ms) | $(median_ms "${m_times[@]}") |"
report_line ""

echo "Wrote $OUT"
exit 0

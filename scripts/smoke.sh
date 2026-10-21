#!/bin/sh
# Smoke-test an installed resolve-merge-conflicts binary.
#
#   scripts/smoke.sh <binary> <expected-version>
#
# Run by the release workflow's `verify` job against the binary that install.sh just installed from
# the freshly built assets, so a broken build is caught before anything is published. It also runs
# locally against `target/release/resolve-merge-conflicts`, where the checks that need a managed install
# (upgrade) are skipped.
#
# Environment honoured:
#   RESOLVE_MERGE_CONFLICTS_API_BASE   when set, `upgrade --check` (latest lookup) is exercised against it
#
# POSIX sh on purpose; no jq, no bash-isms.

set -eu

BIN="${1:?usage: smoke.sh <binary> <expected-version>}"
VERSION="${2:?usage: smoke.sh <binary> <expected-version>}"
VERSION="${VERSION#v}"

pass() { printf '\342\234\223 %s\n' "$1"; }
warn() { printf '! %s\n' "$1" >&2; }
fail() { printf '\342\234\227 %s\n' "$1" >&2; exit 1; }

strip_ansi() { sed 's/\x1b\[[0-9;]*m//g'; }

resolve() {
    # readlink -f is GNU coreutils and macOS 12.3+; realpath covers the rest.
    readlink -f "$1" 2>/dev/null || realpath "$1"
}

tmp="$(mktemp -d)"
# shellcheck disable=SC2064
trap "rm -rf '$tmp'" EXIT INT TERM

# Colour off and git fully isolated from the runner's or developer's global config.
export NO_COLOR=1
export CI="${CI:-1}"
export GIT_CONFIG_GLOBAL=/dev/null
export GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=Smoke GIT_AUTHOR_EMAIL=smoke@example.com
export GIT_COMMITTER_NAME=Smoke GIT_COMMITTER_EMAIL=smoke@example.com
export GIT_TERMINAL_PROMPT=0
export GIT_CONFIG_COUNT=2
export GIT_CONFIG_KEY_0=commit.gpgsign GIT_CONFIG_VALUE_0=false
export GIT_CONFIG_KEY_1=init.defaultBranch GIT_CONFIG_VALUE_1=main

# --- 1. version ---------------------------------------------------------------

actual="$("$BIN" --version)"
[ "$actual" = "resolve-merge-conflicts $VERSION" ] ||
    fail "--version printed '$actual', expected 'resolve-merge-conflicts $VERSION'"
pass "--version reports $VERSION"

# --- 2. help ------------------------------------------------------------------

"$BIN" --help >/dev/null || fail "--help exited non-zero"
"$BIN" upgrade --help >/dev/null || fail "upgrade --help exited non-zero"
pass "--help exits 0"

# --- 3. config and completion --------------------------------------------------

cfgdir="$tmp/xdg"
export XDG_CONFIG_HOME="$cfgdir"
"$BIN" config init >/dev/null || fail "config init exited non-zero"
[ -f "$cfgdir/resolve-merge-conflicts/config.json" ] || fail "config init did not write config.json"
[ -f "$cfgdir/resolve-merge-conflicts/prompt.md" ] || fail "config init did not write prompt.md"
pass "config init writes config.json and prompt.md"

"$BIN" completion zsh | grep -q "resolve-merge-conflicts" || fail "completion zsh printed nothing useful"
pass "completion zsh works"

# --- 4. a real resolve against a throwaway conflicted repo ------------------------
# A fake `claude` stands in for the agent: it rewrites the conflicted file and answers in the
# shape the real one does, so this exercises detection, the prompt, staging and the report.

work="$tmp/work"
mkdir -p "$work"
git -C "$work" init -q
printf 'one\nbase\nthree\n' > "$work/a.txt"
git -C "$work" add -A && git -C "$work" commit -q -m initial
git -C "$work" checkout -q -b feature
printf 'one\nfeature\nthree\n' > "$work/a.txt"
git -C "$work" commit -q -am feature
git -C "$work" checkout -q main
printf 'one\nmain\nthree\n' > "$work/a.txt"
git -C "$work" commit -q -am main
git -C "$work" merge feature >/dev/null 2>&1 && fail "fixture merge unexpectedly succeeded"
head_before="$(git -C "$work" rev-parse HEAD)"

fake="$tmp/claude"
cat > "$fake" <<'FAKE'
#!/bin/sh
cat > /dev/null
printf 'one\nresolved\nthree\n' > a.txt
printf '%s' '{"is_error":false,"result":"```json\n{\"files\":[{\"path\":\"a.txt\",\"strategy\":\"merged\",\"summary\":\"smoke\"}]}\n```"}'
FAKE
chmod +x "$fake"

out="$(RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN="$fake" "$BIN" -c "$work" -a claude -m sonnet -y | strip_ansi)"
case "$out" in
    *"1 of 1 file resolved"*) pass "resolve reports 1 of 1 file resolved" ;;
    *) fail "resolve did not report success. Output:
$out" ;;
esac
[ "$(cat "$work/a.txt")" = "$(printf 'one\nresolved\nthree')" ] || fail "a.txt was not resolved"
git -C "$work" diff --cached --quiet -- a.txt && fail "a.txt was not staged"
[ -z "$(git -C "$work" diff --name-only --diff-filter=U)" ] || fail "a.txt is still unmerged"
[ "$(git -C "$work" rev-parse HEAD)" = "$head_before" ] || fail "HEAD moved: the tool must never commit"
pass "resolve staged the file and left HEAD alone"

# --- 5. upgrade, only meaningful from a managed install ------------------------

case "$(resolve "$BIN")" in
    */versions/*/bin/*)
        check="$("$BIN" upgrade "v$VERSION" --check --json)"
        case "$check" in
            *'"upToDate": true'* | *'"upToDate":true'*) pass "upgrade v$VERSION --check reports up to date" ;;
            *) fail "upgrade --check --json did not report upToDate true: $check" ;;
        esac
        if [ -n "${RESOLVE_MERGE_CONFLICTS_API_BASE:-}" ]; then
            latest="$("$BIN" upgrade --check --json)"
            case "$latest" in
                *'"upToDate": true'* | *'"upToDate":true'*) pass "upgrade --check (latest lookup via $RESOLVE_MERGE_CONFLICTS_API_BASE) reports up to date" ;;
                *) fail "upgrade --check via RESOLVE_MERGE_CONFLICTS_API_BASE did not report upToDate true: $latest" ;;
            esac
        fi
        ;;
    *)
        warn "skipping upgrade checks: $BIN is not inside a managed install (versions/<v>/bin)"
        ;;
esac

pass "smoke test complete"

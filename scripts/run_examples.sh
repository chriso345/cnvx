#!/usr/bin/env bash
#
# Recursively scans an "examples" directory for .rs files, runs each one
# (with --features derived from its doc-comment "Features:" line, if any)
# via `cargo run --example <name>`, and validates stdout against the
# "// Expected output:" comment block found in the file.
#
# Usage:
#   ./run_examples.sh [-d examples_dir] [-m manifest_path] [name ...]
#
#   -d DIR    examples directory to scan (default: examples)
#   -m PATH   path to Cargo.toml / manifest to pass as --manifest-path
#             (default: none, cargo uses the one in the cwd)
#   name ...  optional list of example names (file names without .rs) to
#             run. If omitted, every .rs file found under DIR is run.
#
# Exit code is the number of failed examples (0 means all passed).

EXAMPLES_DIR="examples"
MANIFEST_PATH=""

while getopts ":d:m:h" opt; do
    case "$opt" in
        d) EXAMPLES_DIR="$OPTARG" ;;
        m) MANIFEST_PATH="$OPTARG" ;;
        h)
            sed -n '2,33p' "$0"
            exit 0
            ;;
        \?)
            echo "Unknown option: -$OPTARG" >&2
            exit 2
            ;;
        :)
            echo "Option -$OPTARG requires an argument" >&2
            exit 2
            ;;
    esac
done
shift $((OPTIND - 1))

WANTED_NAMES=("$@")

if [[ ! -d "$EXAMPLES_DIR" ]]; then
    echo "Error: examples directory '$EXAMPLES_DIR' not found" >&2
    exit 1
fi

# Print the features (comma-separated) declared via a
#   //! Features: `foo`, `bar`
# doc-comment line, or nothing if there isn't one.
extract_features() {
    local file="$1"
    local features_line
    features_line="$(grep -m1 -E '^[[:space:]]*//!\s*Features:' "$file" || true)"
    [[ -z "$features_line" ]] && return 0

    local feats=()
    while IFS= read -r feat; do
        [[ -n "$feat" ]] && feats+=("$feat")
    done < <(grep -oE '`[^`]+`' <<< "$features_line" | tr -d '`')

    local IFS=,
    printf '%s' "${feats[*]}"
}

# Print the expected-output block that follows a
#   // Expected output:
# comment, with the leading indentation / "// " prefix stripped, and any
# leading blank separator line (the "//" right after the header) dropped.
extract_expected_output() {
    local file="$1"
    awk '
        BEGIN { found = 0 }
        found == 0 {
            if ($0 ~ /Expected output:/) { found = 1 }
            next
        }
        {
            if ($0 ~ /^[[:space:]]*\/\//) {
                line = $0
                sub(/^[[:space:]]*\/\/[[:space:]]?/, "", line)
                print line
            } else {
                exit
            }
        }
    ' "$file" | awk '
        # Drop leading blank lines only (the separator right after the
        # "Expected output:" header), keep any blank lines that occur once
        # real content has started.
        started == 0 && NF == 0 { next }
        { started = 1; print }
    '
}

name_is_wanted() {
    local name="$1"
    (( ${#WANTED_NAMES[@]} == 0 )) && return 0
    local w
    for w in "${WANTED_NAMES[@]}"; do
        [[ "$w" == "$name" ]] && return 0
    done
    return 1
}

declare -A seen_names
files=()

while IFS= read -r -d '' file; do
    base="$(basename "$file")"
    name="${base%.rs}"

    if [[ -n "${seen_names[$name]+x}" ]]; then
        echo "Error: duplicate example name '$name' found at both '${seen_names[$name]}' and '$file'" >&2
        exit 1
    fi
    seen_names["$name"]="$file"

    name_is_wanted "$name" && files+=("$file")
done < <(find "$EXAMPLES_DIR" -type f -name '*.rs' -print0 | sort -z)

if (( ${#files[@]} == 0 )); then
    echo "No matching .rs files found under '$EXAMPLES_DIR'" >&2
    exit 1
fi

# Sanity check: make sure every requested name was actually found.
for w in "${WANTED_NAMES[@]:-}"; do
    [[ -z "$w" ]] && continue
    if [[ -z "${seen_names[$w]+x}" ]]; then
        echo "Warning: no example named '$w' found under '$EXAMPLES_DIR'" >&2
    fi
done

pass_count=0
fail_count=0
failed_names=()

for file in "${files[@]}"; do
    base="$(basename "$file")"
    name="${base%.rs}"

    features="$(extract_features "$file")"
    expected="$(extract_expected_output "$file")"

    cmd=(cargo run --quiet --example "$name")
    [[ -n "$MANIFEST_PATH" ]] && cmd+=(--manifest-path "$MANIFEST_PATH")
    [[ -n "$features" ]] && cmd+=(--features "$features")

    echo "=== $name ($file) ==="
    if [[ -n "$features" ]]; then
        echo "  features: $features"
    fi
    echo "  cmd: ${cmd[*]}"

    actual="$("${cmd[@]}" 2>/tmp/run_examples_stderr.$$)"
    status=$?

    if (( status != 0 )); then
        echo "  FAILED (exit code $status)"
        echo "  --- stderr ---"
        sed 's/^/    /' /tmp/run_examples_stderr.$$
        fail_count=$((fail_count + 1))
        failed_names+=("$name")
        echo
        rm -f /tmp/run_examples_stderr.$$
        continue
    fi
    rm -f /tmp/run_examples_stderr.$$

    if [[ -z "$expected" ]]; then
        echo "  SKIPPED validation (no 'Expected output:' block found)"
        echo "  --- actual output ---"
        sed 's/^/    /' <<< "$actual"
        echo
        continue
    fi

    # Trim trailing whitespace/newlines from both sides before comparing.
    norm_actual="$(printf '%s' "$actual" | sed -e 's/[[:space:]]*$//')"
    norm_expected="$(printf '%s' "$expected" | sed -e 's/[[:space:]]*$//')"

    if [[ "$norm_actual" == "$norm_expected" ]]; then
        echo "  PASS"
        pass_count=$((pass_count + 1))
    else
        echo "  FAILED (output mismatch)"
        echo "  --- diff (expected vs actual) ---"
        diff <(printf '%s\n' "$norm_expected") <(printf '%s\n' "$norm_actual") | sed 's/^/    /'
        fail_count=$((fail_count + 1))
        failed_names+=("$name")
    fi
    echo
done

echo "==================================================="
echo "Ran $((pass_count + fail_count)) example(s): $pass_count passed, $fail_count failed"
if (( fail_count > 0 )); then
    echo "Failed: ${failed_names[*]}"
fi

exit "$fail_count"

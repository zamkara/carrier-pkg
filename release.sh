#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
readonly PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly VERSION_FILE="$SCRIPT_DIR/VERSION"

# ── Color ────────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    readonly BOLD='\033[1m'; readonly GREEN='\033[1;32m'; readonly CYAN='\033[1;36m'
    readonly YELLOW='\033[1;33m'; readonly RED='\033[1;31m'; readonly RESET='\033[0m'
else
    readonly BOLD=''; readonly GREEN=''; readonly CYAN=''; readonly YELLOW=''; readonly RED=''; readonly RESET=''
fi

info()  { echo -e "${CYAN}==>${RESET} $*"; }
ok()    { echo -e " ${GREEN}\u2714${RESET} $*"; }
warn()  { echo -e " ${YELLOW}\u26a0${RESET} $*"; }
err()   { echo -e " ${RED}\u2717${RESET} $*"; }

# ── Help ──────────────────────────────────────────────────────────────────────
usage() {
    cat <<EOF
Usage: $(basename "$0") [bump] [options]

Bump:  major | minor | patch | <semver>
       Default: patch

Options:
  --dry-run    Show what would be done, no changes
  --help, -h   Show this help

Examples:
  ./release.sh           # bump patch → build → tag → release
  ./release.sh minor     # bump minor
  ./release.sh 0.2.0     # set explicit version
  ./release.sh --dry-run # dry run
EOF
    exit 0
}

# ── Parse args ────────────────────────────────────────────────────────────────
BUMP="patch"
DRY_RUN=false

for arg in "$@"; do
    case "$arg" in
        --help|-h) usage ;;
        --dry-run) DRY_RUN=true ;;
        major|minor|patch|[0-9]*.[0-9]*.[0-9]*) BUMP="$arg" ;;
        *) err "unknown argument: $arg"; usage ;;
    esac
done

# ── Semver helpers ────────────────────────────────────────────────────────────
semver_parse() {
    local v="$1"; v="${v#v}"; v="${v#V}"
    IFS='.' read -r major minor patch <<< "$v"
    echo "$((major)) $((minor)) $((patch))"
}

semver_bump() {
    local current="$1" bump="$2"
    read -r ma mi pa <<< "$(semver_parse "$current")"
    case "$bump" in
        major) echo "$((ma+1)).0.0" ;;
        minor) echo "$ma.$((mi+1)).0" ;;
        patch) echo "$ma.$mi.$((pa+1))" ;;
        *)     echo "$bump" ;;  # explicit version
    esac
}

# ── Read / write version ──────────────────────────────────────────────────────
read_current_version() {
    # Prefer VERSION file, fallback to Cargo.toml
    if [ -f "$VERSION_FILE" ]; then
        cat "$VERSION_FILE"
    else
        grep '^version' "$SCRIPT_DIR/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)".*/\1/'
    fi
}

write_version() {
    local ver="$1"
    local main_rs="$SCRIPT_DIR/src/main.rs"
    echo "$ver" > "$VERSION_FILE"
    sed -i "s/^version = \".*\"/version = \"$ver\"/" "$SCRIPT_DIR/Cargo.toml"
    if grep -q 'println!("carrier ' "$main_rs" 2>/dev/null; then
        sed -i "s/\(\"carrier \)[0-9]*\.[0-9]*\.[0-9]*\(\"\)/\1$ver\2/" "$main_rs"
    fi
    info "version $ver written to VERSION, Cargo.toml, main.rs"
}

# ── Build ──────────────────────────────────────────────────────────────────────
do_build() {
    info "building carrier v$NEW_VER (release)..."
    if ! $DRY_RUN; then
        (cd "$SCRIPT_DIR" && cargo build --release)
    fi
    ok "build complete"
}

# ── Package ────────────────────────────────────────────────────────────────────
do_package() {
    local ver="$1"
    local archive="carrier-v${ver}-x86_64-linux.tar.gz"

    info "packaging $archive..."
    if ! $DRY_RUN; then
        local dist_dir="$SCRIPT_DIR/dist"
        mkdir -p "$dist_dir"
        tar czf "$dist_dir/$archive" -C "$SCRIPT_DIR/target/release" carrier
        ok "created $dist_dir/$archive ($(du -h "$dist_dir/$archive" | cut -f1))"
    else
        ok "would create dist/$archive"
    fi
}

# ── Git tag + release ─────────────────────────────────────────────────────────
do_git_release() {
    local ver="$1"
    local tag="v$ver"
    local archive="carrier-v${ver}-x86_64-linux.tar.gz"
    local dist_dir="$SCRIPT_DIR/dist"

    # Check if we're in a git repo
    if ! git rev-parse --git-dir &>/dev/null 2>/dev/null; then
        warn "not a git repository — skipping git tag and GitHub release"
        return
    fi

    # Check for uncommitted changes
    if ! git diff --quiet 2>/dev/null || ! git diff --cached --quiet 2>/dev/null; then
        warn "uncommitted changes — commit first or skip git operations"
        warn "  git add -A && git commit -m \"release v$ver\""
        return
    fi

    if $DRY_RUN; then
        ok "would create git tag $tag"
        if command -v gh &>/dev/null && gh auth status &>/dev/null 2>&1; then
            ok "would create GitHub release $tag with dist/$archive"
        else
            warn "gh CLI not available — would skip GitHub release"
        fi
        return
    fi

    # Create git tag
    if git tag | grep -q "^$tag$"; then
        warn "tag $tag already exists — skipping tag"
    else
        info "creating git tag $tag..."
        git tag -a "$tag" -m "carrier v$ver"
        ok "tagged $tag"
    fi

    # Create GitHub release if gh CLI is available
    if command -v gh &>/dev/null && gh auth status &>/dev/null 2>&1; then
        if [ -f "$dist_dir/$archive" ]; then
            info "creating GitHub release $tag..."
            if gh release view "$tag" &>/dev/null 2>&1; then
                warn "release $tag already exists — uploading assets"
                gh release upload "$tag" "$dist_dir/$archive" --clobber
            else
                gh release create "$tag" \
                    --title "carrier v$ver" \
                    --notes "See [CHANGELOG](CHANGELOG.md) for details." \
                    "$dist_dir/$archive"
            fi
            ok "GitHub release $tag created"
        else
            warn "archive not found at $dist_dir/$archive — build and package first"
        fi
    else
        warn "gh CLI not available or not authenticated — skipping GitHub release"
    fi
}

# ── Main ───────────────────────────────────────────────────────────────────────
echo -e "${BOLD}carrier release${RESET}"

CURRENT_VER=$(read_current_version)
NEW_VER=$(semver_bump "$CURRENT_VER" "$BUMP")

echo "  version:  ${CYAN}$CURRENT_VER${RESET} → ${GREEN}$NEW_VER${RESET}"
echo "  dry-run:  ${DRY_RUN:-false}"
echo

if $DRY_RUN; then
    info "dry-run mode — no files will be changed"
fi

# 1. Update version files
info "updating version files..."
if ! $DRY_RUN; then
    write_version "$NEW_VER"
    ok "version $NEW_VER written to VERSION, Cargo.toml, main.rs"
else
    ok "would write version $NEW_VER"
fi

# 2. Build
do_build

# 3. Package
do_package "$NEW_VER"

# 4. Git tag + GitHub release
do_git_release "$NEW_VER"

echo
echo -e "${GREEN}\u2714 carrier v$NEW_VER released${RESET}"

if $DRY_RUN; then
    echo -e "  ${YELLOW}(dry run — no changes were made)${RESET}"
fi
echo
echo "  artifacts: dist/carrier-v${NEW_VER}-x86_64-linux.tar.gz"
echo "  tag:       v$NEW_VER"
echo "  to push:   git push origin v$NEW_VER"

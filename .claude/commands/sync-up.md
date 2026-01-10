# Sync Up Command

Analyze recent changes and check if documentation and dev scripts need updates.

## Instructions

When this command is invoked, perform the following analysis:

### 1. Gather Recent Changes

First, collect information about recent changes:

```bash
# Recent commits (last 10)
git log --oneline -10

# Check for uncommitted changes
git status --short

# Show files changed in recent commits
git diff --name-only HEAD~5..HEAD 2>/dev/null || git diff --name-only HEAD~3..HEAD

# List current branches
git branch -a --list | head -20
```

### 2. Analyze Tmuxinator Scripts

Check if `scripts/nullblock-dev-mac.yml` and `scripts/nullblock-dev.yml` need updates:

**Look for:**
- New services added to `svc/` that aren't in tmuxinator windows
- Port changes in services that aren't reflected in health checks
- New scripts in `scripts/` that should be integrated
- Services removed that are still in tmuxinator

**Files to check:**
- `scripts/nullblock-dev-mac.yml`
- `scripts/nullblock-dev.yml`
- `justfile` (for new commands)

### 3. Analyze mdBook Documentation

Check if `docs-internal/` needs updates:

**Look for:**
- New services in `svc/` without documentation pages
- Changes to CLAUDE.md that should be reflected in the book
- New API endpoints not documented in `reference/api.md`
- Port changes not reflected in `ports.md`
- Architecture changes not in `architecture.md`

**Files to check:**
- `docs-internal/src/SUMMARY.md` (navigation)
- `docs-internal/src/services/*.md`
- `docs-internal/src/ports.md`
- `docs-internal/src/reference/api.md`

### 4. Report Findings

Present findings in this format:

```
## Sync-Up Report

### Recent Activity
- [Summary of recent commits]
- [Any uncommitted changes]

### Tmuxinator Updates Needed
- [ ] Item 1
- [ ] Item 2
(or "No updates needed")

### Documentation Updates Needed
- [ ] Item 1
- [ ] Item 2
(or "No updates needed")

### Recommended Actions
1. [Action 1]
2. [Action 2]
```

### 5. Offer to Make Changes

After presenting the report, ask if the user wants you to:
- Update tmuxinator scripts
- Update documentation
- Both
- Neither (just informational)

Only make changes if the user confirms.

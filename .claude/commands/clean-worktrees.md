# Clean Worktrees - Remove Merged Module Worktrees

Remove worktrees for branches that have been merged and deleted on the remote.

## Usage

```
/clean-worktrees              # Clean all merged worktrees
/clean-worktrees <module>     # Clean specific module worktree
/clean-worktrees --list       # List all active worktrees
/clean-worktrees --force      # Remove all worktrees (even unmerged)
```

## Process

### 1. Fetch Latest Remote State

```bash
git fetch --prune
```

### 2. List Active Worktrees

```bash
source .claude/lib/workflow-state.sh
worktree_list
```

**Output:**
```
.worktrees/session   abc1234 [feat/session]
.worktrees/auth      def5678 [feat/auth]
.worktrees/cycle     (bare)
```

### 3. Check for Merged Branches

For each worktree, check if its branch has been merged and deleted:

```bash
# Get branches marked as [gone] (deleted on remote after merge)
git branch -vv | grep '\[.*: gone\]'
```

### 4. Clean Merged Worktrees

```bash
worktree_cleanup_merged
```

**Output:**
```
Cleaning up merged worktree: session (feat/session)
Cleaning up merged worktree: auth (feat/auth)
Cleaned 2 merged worktree(s)
```

### 5. Manual Cleanup (Specific Module)

To remove a specific worktree regardless of merge status:

```bash
worktree_remove "<module>"
```

## Integration with PR Workflow

After a PR is merged on GitHub:

1. GitHub deletes the branch (if configured)
2. Run `git fetch --prune` to update local tracking
3. Run `/clean-worktrees` to remove stale worktrees

### Automatic Cleanup

The `/dev` skill checks for merged worktrees at startup:

```bash
# In dev.md workflow init
worktree_cleanup_merged 2>/dev/null || true
```

## Options

| Option | Description |
|--------|-------------|
| `--list` | Show all active worktrees without cleaning |
| `--force` | Remove all worktrees (use with caution) |
| `--dry-run` | Show what would be cleaned without removing |

## Examples

### List Worktrees
```
> /clean-worktrees --list

Active module worktrees:
  📁 .worktrees/session   → feat/session (3 ahead)
  📁 .worktrees/auth      → feat/auth [PR #42 merged, ready to clean]
  📁 .worktrees/cycle     → feat/cycle (working)

Total: 3 worktrees
```

### Clean After Merge
```
> /clean-worktrees

Fetching remote state...
Checking for merged branches...

Found 1 merged worktree:
  📁 .worktrees/auth → feat/auth [PR #42 merged]

Cleaning...
✅ Removed: .worktrees/auth

1 worktree cleaned.
```

### Force Clean
```
> /clean-worktrees --force

⚠️  This will remove ALL worktrees, including unmerged work!

Active worktrees:
  📁 .worktrees/session (3 uncommitted changes)
  📁 .worktrees/auth (clean)

Proceed? [y/N]: y

Removing .worktrees/session...
Removing .worktrees/auth...

✅ Removed 2 worktrees.
```

## See Also

- `/dev` - Feature-driven development workflow
- `git worktree list` - Native git worktree listing
- `git worktree prune` - Clean stale worktree metadata

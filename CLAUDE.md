# Claude Code Guidelines

## Git

- **NEVER bypass GPG commit signing.** If signing fails, stop and let the user handle it. Do not use `--no-gpg-sign`, `-c commit.gpgsign=false`, or any other workaround.
- **Create a PR for each issue.** Don't commit directly to main. For each GitHub issue:
  1. Create a feature branch (e.g., `feat/issue-description`)
  2. Make commits on that branch
  3. Open a PR linking the issue
  4. Merge after review

# Claude Code Guidelines

## Git

- **NEVER bypass GPG commit signing.** If signing fails, stop and let the user handle it. Do not use `--no-gpg-sign`, `-c commit.gpgsign=false`, or any other workaround.

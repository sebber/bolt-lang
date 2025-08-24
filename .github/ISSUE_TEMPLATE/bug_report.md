---
name: Bug Report
about: Report a bug in the Bolt compiler or language
title: "[BUG] "
labels: bug
assignees: ''
---

## Bug Description
A clear and concise description of what the bug is.

## To Reproduce
Steps to reproduce the behavior:

1. Create a file with this Bolt code:
   ```bolt
   // Your Bolt code here
   ```

2. Compile with: `./target/debug/bolt file.bolt -o output`
3. Run with: `./out/debug/output`
4. See error: `error message here`

## Expected Behavior
A clear and concise description of what you expected to happen.

## Actual Behavior
What actually happened instead?

## Environment
- OS: [e.g., Ubuntu 22.04, macOS 14.0, Windows 11]
- Bolt compiler version: [e.g., commit hash or release version]
- Rust version: [output of `rustc --version`]

## Additional Context
- Generated C code (if relevant)
- Compiler output/error messages
- Any other context about the problem

## Minimal Reproduction
Please provide the smallest possible Bolt program that reproduces the issue:

```bolt
// Minimal code that reproduces the bug
```
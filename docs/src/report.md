# The report

```
Resolution report
claude (sonnet) · merge · 2 of 2 files resolved

  ✓ src/app.rs   both modified  merged both  +6 −2  staged
      kept the new retry logic and the renamed config field
  ✓ Cargo.lock   both modified  took theirs  +40 −38  staged
      regenerated lockfile content from the incoming side

Next
  Review what was resolved:  git diff --staged
  When you are happy:        git merge --continue
Nothing was committed or pushed; that is left to you.
```

If anything looks off it appears under **Look closer at**: the agent claiming a file was handled when
markers remain, claiming one strategy when the content looks like another, skipping a file in its
summary, mentioning a file that was never in conflict, editing files outside the conflict, or moving
`HEAD`. `--json` prints the same data, plus the mismatches, for scripts.

# libass tag compatibility: reproducible TDD

Base: upstream `5c944eaaff75d4fa957d965964e6c039a183b6b8`.
This branch changes tag boundaries and recovery, not rendering or style lookup.

## Commit contracts

| Commit | Phase | Regression result with `--features full` |
| --- | --- | --- |
| `8a63527` | RED: empty override recovery | 4 fail, 1 pass |
| `c7f8e0a` | GREEN: preserve the next backslash, recover on UTF-8 boundaries | 5 pass |
| `e60b205` | RED: unwrapped color/alpha arguments | 4 fail |
| `ba6bf51` | GREEN: separate built-in color names from arguments | 9 pass across both test files |

The failures were observed before implementation. These four committed states
were also independently checked out and rerun after organizing the commits.
The final tests include additional compatibility assertions added during review.
RED commits intentionally fail; the branch head must remain GREEN.

## Reproduce

Use a disposable detached worktree at each commit, then run:

```sh
cargo test -p ass-core --no-default-features --features full --test tag_recovery_tdd
```

At the color RED commit and later:

```sh
cargo test -p ass-core --no-default-features --features full --test color_tag_tdd
```

Empty recovery tests exercise standard and registry parsers, repeated/trailing
backslashes, following font/reset/bold/drawing tags, multibyte malformed names,
and surrounding Unicode/parenthesized arguments. Malformed input still emits
diagnostics; callers decide whether an isolated empty override is acceptable.

Color tests cover all ten color/alpha names with nine argument forms per parser,
following font tags, clipping, other tags, and registered extension precedence
for successful, ignored, and failed handlers. Arguments remain borrowed and
unmodified. The parser does not evaluate color values or rewrite subtitle text.

## Validation

The supported debug test matrix passed: workspace full and full/simd-full,
plus core/editor minimal and minimal/nostd. Workspace full and core/editor
minimal/nostd all-target clippy passed with warnings denied. Formatting,
workspace full release build, and parser benchmark smoke tests passed.

LLVM instrumentation measured 100% line coverage for the color boundary helper
and over 94% for each modified parser module. This is changed-module coverage,
not a claim of greater than 90% coverage for the entire workspace.

WASM integration and real-subtitle regressions are tracked by the consuming
assfonts-rs repository, pinned to this fork's complete commit SHA.

# Declared ply

A public note based on the author of [biryukovmaxim/vprog-tictactoe](https://github.com/biryukovmaxim/vprog-tictactoe). It is not a pull request, not a comment, and not a node.

The long reading is [VISION.md](VISION.md). The code is the part you can run.

## What

Two pure functions for the same tic-tac-toe position.

`guest_act` follows the guest at tip `098be674`. A cell from the seat that is waiting goes on a ring of four. After the explicit cell, `drain_pending` keeps applying that ring, including as the first ply of the next round, while the match is still open. When the match ends, the ring is cleared first, so a one-round game does not open another round.

`land_batch` takes every attempt in one declaration and lands one cell. The cell that would have opened the next round comes back `Retry`. The next board stays empty. A second try at the landed cell is `SecondSpend`. An attempt closer than 25 sequence steps to the previous ply is `Immature`.

`match_outcome` is the guest's clinch. A seat that is ahead by more than the rounds still left wins, and those leftover rounds are not played. The score is by seat, because X and O swap each round. Creator mark is X, mark byte 1.

Nothing here submits a transaction, loads a wallet, or links the guest.

## Why

A client that cannot see which attempt landed will send the cell again, and will sometimes send the next cell in the same breath. The guest ring can turn that extra cell into the opening of the next round. One declared ply makes the landing visible: one blue mark, and a named reason for every other attempt.

That rule sits in a larger split. The coin's covenant is the lock a Kaspa full node already runs. The guest is the rules that do not fit the script meter. [Topic 387](https://research.kas.pa/t/concrete-proposal-for-a-synchronously-composable-verifiable-programs-architecture/387) asks for a further step this match does not take: one transaction that reads one program's account and writes another's, with each program proving only its own segment. Michael Sutton, on 26 Sep 2026, called the public demo a standalone verifiable program that composes through layer 1, and said the yellow-paper design still needs more layer-1 work.

## Sources

Pins read on 29 Sep 2026. The full list is in [VISION.md](VISION.md).

- [biryukovmaxim/vprog-tictactoe](https://github.com/biryukovmaxim/vprog-tictactoe) `098be674`. `guest/src/program/action/game.rs` (`commit_explicit`, `drain_pending`, `apply_create_game`) and `guest/src/program/rules.rs` (`match_outcome`). `Cargo.toml` and `Cargo.lock` pin vprogs branch `fix/settlement-watch-state-neutral` at `07d486dd`. The README setup line names `fix/settlement-watch-wedge`.
- [kaspanet/vprogs](https://github.com/kaspanet/vprogs) master `f9b84a8` (28 Jul 2026, no release). `07d486dd` is the settlement-watch tip, not a release. Deposit binding: `zk/abi/src/delegate_script.rs` and `zk/abi/src/batch_processor/verifier.rs`.
- [kaspanet/kips](https://github.com/kaspanet/kips) `e4ae233`. KIP-16, KIP-17, KIP-20, and KIP-21 are Active. KIP-21 authors: Michael Sutton, Maxim Biryukov, Hans Moog.
- [research.kas.pa topic 387](https://research.kas.pa/t/concrete-proposal-for-a-synchronously-composable-verifiable-programs-architecture/387), with 323, 347, 375, 407, 410, and 411. A thread is not a KIP.
- Michael Sutton, 26 Sep 2026, post [2103892665435578727](https://x.com/michaelsuttonil/status/2103892665435578727).
- [kaspanet/silverscript](https://github.com/kaspanet/silverscript) tag v1.0.0 `3ed9733`. [argent-lang/argent](https://github.com/argent-lang/argent) master `e76ee07`, no tags.
- [STP-KAS/tn10-vprogs-final-verdict](https://github.com/STP-KAS/tn10-vprogs-final-verdict) and [STP-KAS/kaspa-master-file](https://github.com/STP-KAS/kaspa-master-file) `34ecaf7`.

## What is interesting

The lock and the rules are different programs. KIP-17 and KIP-20 let the coin constrain its next spend and keep a lineage id. KIP-16 lets that coin check a proof instead of replaying the board. KIP-21 lets the proof follow the game's own lane. The guest then owns the board, the seat score, and the clinch. A full node never has to learn what three in a row means.

The interesting bug is smaller, and it is the one `cargo run --example boards` prints. Same two cells. The guest plays the second cell on a fresh board. The batch does not.

## Build, test, try

Rust stable. No extra crates.

```
cargo test
cargo run --example boards
```

`cargo test` runs 9 tests: the clinch numbers, the ring of four, the occupied-cell reject, the one-round clear, and the batch that refuses the next round.

`cargo run --example boards` prints this. Before the last ply the creator, as X, is one move from the column 2-4-6:

```
O O X
. X .
. . .
```

Guest rule. The joiner queued cell 8, then the creator played 6. The win cleared the board and cell 8 opened the next round as X. Creator wins 1. Round index 1. The match is still open.

```
. . .
. . .
. . X
```

Batch rule. The same two cells are one declaration. Blue is creator X on 6. Red is joiner cell 8, `Retry`. The next board stays empty. Same score, same open match.

```
. . .
. . .
. . .
```

Guest rule on one round. The same queue is cleared because the match ended. The board stays empty. Over: creator. Clinch: false, because no leftover round was skipped.

## Inconsistencies

These are disagreements inside the sources. This crate does not patch them.

| Where | What the files say |
| --- | --- |
| Round boundary | On a best of three, `drain_pending` plays a queued cell as the first ply of the next round. On one round, `end_match_if_decided` clears the ring first and the next board stays empty. Same path, two results. |
| Zero rounds, zero stake | `apply_create_game` rejects `rounds == 0`. `write_game` stores `rounds_total` and `stake` with no zero check. `checked_sub(0)` succeeds, so a stake of 0 is a successful debit. This crate refuses a zero round count and does not model stake. |
| One bad cell | An occupied explicit cell rejects the whole guest action before any write. `land_batch` marks that attempt red and can still land a different cell in the same declaration. |
| Which vprogs commit | At `098be674`, the README tells you to check out `fix/settlement-watch-wedge`. `Cargo.toml` and `Cargo.lock` pin `fix/settlement-watch-state-neutral` at `07d486dd`. |
| Release | kaspanet/vprogs has no release. Master is `f9b84a8`. The name `release-candidate` on `07d486dd` is a branch, not a release. |
| What the demo is | Topic 387 is synchronous composition across two programs. Sutton's 26 Sep note calls the public demo a standalone verifiable program. The match in this crate is one program. |
| Script version | SilverScript's tag is v1.0.0 (`3ed9733`). The compiler still stamps `compiler_version` 0.1.0, and `pragma silverscript ^1.0.0` is rejected. Argent's own README still says it is not ready for general production, and the repo has no tag. A tagged SilverScript does not make Argent ready. |
| Node pin | The guest's Cargo.toml pins rusty-kaspa `eb0a856`. The published node release v2.1.0 is `01b532e8`. |
| Campaign counters | The 25 Sep status file is behind the finished log. Recounted 29 Sep: 1,842 accepted sends, 885,963 send failures, 33,771 game failures, 0 finished games. 919,418 failures are HTML where the client expected JSON. |

The hosted page and this crate are not "vProgs live". Topic 387 is not a product.

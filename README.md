# Ply for Maxim

A private note for the author of [biryukovmaxim/vprog-tictactoe](https://github.com/biryukovmaxim/vprog-tictactoe). It is not a pull request, not a comment, and not a node.

The long reading is [VISION.md](VISION.md). The code is the short part.

`guest_act` follows the guest at tip `098be674`: a cell from the seat that is waiting goes on a ring of four, and `drain_pending` can play that cell as the first ply of the next round while the match is still open. A finished match clears the ring first, so a one-round game does not do this.

`land_batch` declares every attempt and lands one. The cell that would have opened the next round comes back `Retry`. The board stays empty.

```
cargo test
cargo run --example boards
```

The example prints three boards. The guest board has an X in the corner that the batch board does not.

Checked against `guest/src/program/action/game.rs` and `guest/src/program/rules.rs` at `098be674`. The local checkout at `93b7590` was not moved. Nothing here links the guest, a prover, or a wallet.

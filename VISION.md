# What vProgs is asking for, and what this guest already is

Written for the tic-tac-toe guest. Pins are the ones read on 29 Sep 2026. A forum thread is not a KIP. A release-candidate branch is not a release. The hosted page is not the yellow paper.

## The goal

Kaspa already lets many blocks exist at once and then puts them in order. A program that wants its own sequencer throws that away and then has to invent a bridge back.

The concrete proposal, research.kas.pa topic 387 (hashdag with Michael Sutton, 7 Aug 2025), asks for something narrower than "a smart contract platform":

- Accounts hold state. A vProg owns its accounts and is the only program that may write them.
- A transaction says, up front, which accounts it will read and which of those it will write.
- Each vProg is its own small zero-knowledge machine, with its own L1 covenant and its own gas.
- One transaction may read an account of program A and write an account of program B, and both writes land or neither does.
- Program B does not trust program A's execution, and does not trust that A kept its history.
- The proof publishes a state root to L1. That root is what stops one program's scope from swallowing the other program's whole history.
- L1 nodes track the declarations and the gas. They do not run the programs.

The sentence he uses for the failure mode is about liquidity. If programs can only compose inside one rollup, the money moves into that rollup and then will not come back out to meet a different program. Synchronous composability is his answer to that.

Topic 407 is the cost model of that idea on one sequencer: scope is a walk backward from the declared reads, stopping at proof anchors. Topic 410 adds the sovereignty rule: a transaction that forces work it did not pay for is a bug, a failed write still occupies its declared slot by copying the old value, and dynamic "just run it and see what it touched" is rejected because then every node would have to execute. Topic 411 says a scope that needs bytes below the pruning point is forbidden. Topic 323 is the earlier sketch of stitching each program's proof instead of proving the whole transaction as one program. Topic 347 is the tension he is willing to keep: a parallel L1 includes a transaction before the order is final, so a proof of a cross-program call waits until that order exists.

He asked for a yellow paper by the end of August 2025. Two questions in the thread, the largest practical problem and what happens to a lost witness, have no answer there.

## What it changes

On a covenant-only chain, the coin carries its next rule, and every full node runs that rule. The rule has to fit the script meter. On a classic rollup, one operator's machine runs many applications, they compose easily with each other, and they compose with the outside world through a bridge and a delay.

The proposal changes the boundary. The full node stops being the interpreter of the application. It becomes the checker of a declaration, a payment, and a proof. The application interpreter is the vProg, and it is not shared. Composition is a property of the transaction's declared accounts, not a property of living in the same virtual machine.

What the running code changes, today, is smaller. kaspanet/vprogs master `f9b84a8` (28 Jul 2026) is an early prototype: a scheduler, a runtime, storage, and a runner. The branch called `release-candidate` at `07d486dd` (27 Sep 2026) is the tip of a settlement-watch fix. It is not a release. There is no release.

Michael Sutton, 26 Sep 2026, on the public demo: it is not the full yellow-paper syncompo design, that still needs more L1 work. It is a standalone verifiable program that can compose through L1. He calls that a based zk app. He also says calling it a vprog is fair, because it is the same codebase on the way there.

That is the accurate size of the change that exists. One program. Its own covenant. Its actions are transactions on an L1 lane. A proof later moves the locked coins. A second program in the same transaction is not what that demo does.

## Why it is interesting

The interesting part is the split between the lock and the rules.

A full node can already check a covenant. After Toccata (DAA score 474165565) the script engine has KIP-17 introspection, so a coin can constrain the transaction that spends it, and KIP-20 covenant ids, so a lineage of outputs cannot be forged from a lookalike script. KIP-16 is the opcode that checks a zero-knowledge proof. KIP-21, whose authors are Sutton, Maxim Biryukov, and Hans Moog, replaces one global sequencing commitment with a commitment per active lane, so a prover's work follows that lane's activity. The KIP itself says a vProg can later be modeled as a lane, and that this commitment is meant to stay compatible with a future computation-DAG spec. Status of all four, at kips `e4ae233`, is Active.

That is a real base. A program that is too big for the script meter can still put its money behind a coin every Kaspa node already understands. The node checks the covenant id, the proof, and the lane. The program checks the game. Neither has to pretend to be the other.

The tic-tac-toe guest is a clean example of that split, and only of that split. Two players lock a stake. The guest owns the accounts, the board, the rounds, and the pot. Settlement pays the pot out on L1. The runner at `07d486dd` puts the carriers on a KIP-21 user lane (`SubnetworkId::from_namespace` of the lane id) and leaves bootstrap and settlement on the native subnetwork. The deposit address is tied to the covenant id by the redeem script on chain. The prover does not invent that address. In `zk/abi/src/batch_processor/verifier.rs` the comment on `assert_declared_deposit_hash` says a program may pay a deposit anywhere, and binding that address to the covenant id is the covenant's job. The proof only checks that the executed batch and the declared pin are the same pin. The delegate script bytes live in `zk/abi/src/delegate_script.rs`: a prefix, the 32-byte covenant id, a suffix. They are raw script, shared so the guest, the permission sweep, and the settlement covenant hash the same bytes.

The other interesting part is scope. If the proof anchors the state, a reader does not replay every earlier game. It walks back to the last anchor. A quiet program stays out of a busy program's proof. KIP-21 is the L1 half of that idea already, for lanes. The account-level half is what topic 407 calculates and what the demo does not yet need, because the demo has one program.

There is also a small interesting rule already inside the guest, which this repo's tests copy. `match_outcome` ends the match when one seat is ahead by more than the rounds still left. The letters swap each round, and the score is by seat. That is ordinary game logic. It is also the right shape for a sovereign account: the program decides a fact the L1 coin never inspects, and the coin only sees the outcome inside a proof.

## Why it is not interesting

It is not interesting if the thing you want is one game, or one escrow, or one name.

A two-player pot already fits a covenant. kas-odds is a SilverScript two-player game: each side locks at least 1 KAS and two committed bits decide the winner. Its own page calls the default profile a testnet-10 MVP. Kurrent is an eltoo-style two-party channel on KIP-17 and KIP-20, local-devnet, and its author thinks the two-party case does not need KIP-21. Neither of those is a vProg, and neither needs one.

It is not interesting as a throughput story. Topic 247 says provers are two or three orders of magnitude heavier than execution, and a gas cap that lives only inside the program lets L1 include transactions the program will ignore. The Testnet 10 storm agrees with the shape of that warning. The public verdict (26 Sep 2026) is that the chain kept producing blocks and accepting transactions, the vProgs client lost the flood, and the guest in exec mode refused every impossible debit it was shown (0 of 3,518, then 0 of 4,291) with proofs off. Upstream tic-tac-toe finished 5 games, then 8. The private campaign against the hosted lane finished 0. Its raw log, recounted 29 Sep, is 1,842 accepted sends, 885,963 send failures, 33,771 game failures, and no finished game. The dominant reason, 919,418 times, is HTML where the client expected JSON.

It is not interesting as data availability. Topic 375 states the attack: if L1 prunes and only a volunteer kept the program's bytes, an untracked history can be composed into someone else's state by the only person who still has it. Max's public note (15 Sep 2026) says data availability moved to the vprog level, L1 holds the digest, and peer-to-peer download and initial block download are not implemented in the framework. A later note (25 Sep) says that today you run the node that proves and settles, and a mode without proving exists for the data. That is an operator choice, not a protocol answer to topic 375.

It is not interesting as a finished composition design. The yellow paper's open piece, which topic 387 names, is a standard proof that the covenant really locks the program you think it locks. He calls that a zk-SBOM. It is still under research. Sutton's 26 Sep sentence says the missing piece is more L1, not more guest code.

And it is not interesting as a product claim. Master has no release. The demo's README setup line and the Cargo pin have disagreed about which vprogs commit to check out. The master file's do-not-weld line exists because those slips are easy: tictactoe is not vProgs live, the hosted URL is not vProgs live, the release-candidate tip is not a release.

## Is it new

The pieces are not new.

Zero-knowledge proofs of a program are a decade of work. A rollup that posts a state root and lets L1 check a proof is how zk rollups already work. "Based" sequencing, where the L1 order is the program's order, is an existing design response to a single sequencer. Covenants are older than Kaspa's script: Bitcoin has spent years on the same wish, and Kaspa shipped a version of it. Declared reads and writes are how Solana and Sui schedule, which topic 410 says out loud, and it says why it rejects Aptos-style detection. Stitching proofs of different segments is what topic 323 compares to LegoSNARK (eprint 2019/142).

What the proposal claims as its own is the combination. Sovereign machines, account-level atomic composition, an L1 that stays ignorant of execution, and a proof scope that stops at anchors, on a blockDAG that already ordered the transaction before any program ran. That combination is not shipped. The shipped Kaspa part is the covenant and lane machinery a based zk app can use while the combination is still a paper.

So the honest answer is: new as a research claim about composition, familiar as a way to lock coins behind a proof. The tic-tac-toe guest is the familiar half.

## Covenants, SilverScript, Argent

These four are stacked, and they are not substitutes.

A covenant is the rule on the coin. KIP-17 gives the script the spending transaction, so the rule can say what the next output must look like. KIP-20 gives that lineage a 32-byte id that consensus tracks. A lookalike script with a forged state does not inherit the id. Topic 258 and topic 293 are the bridge version of the same worry: the id is how L1 names the program without reading the program.

SilverScript is the language that compiles to that script. v1.0.0 is the tag `3ed9733` (9 Sep 2026). The compiler binary still stamps `compiler_version` 0.1.0, and `pragma silverscript ^1.0.0` is rejected, so the tag and the pragma are not the same number. The v1.0.0 meter, from the changes merged in that tag, includes a 244-item stack, a 250,000-byte signature script, a 10,000-iteration budget per function, and a required state field on a covenant. Open issues remain on that tag, including the compute-budget estimate and the `state:` layout bug. A contract that fits this meter is an L1 application. kas-odds is that kind of application.

Argent is a language in front of SilverScript. It compiles `.ag` to `.sil`. Its authors still say it is under active development and not ready for general production. The GitHub has no release and no tag. Master on 26 Sep was `e76ee07`. A stable SilverScript tag does not make Argent ready. An expert reading the generated `.sil` is a narrower path than handing someone the `.ag`. Issue 64, still open on that read, says `argent-runtime` does not build against rusty-kaspa v2.1.0.

vProgs sits behind that stack, not beside it as a rival. The coin rule that must be true for every full node is a covenant, written in script, which SilverScript and later Argent are for. The transition that does not fit 10,000 iterations, or that you do not want every node to re-execute, is a guest. KIP-16 is the hinge: the covenant checks a proof instead of replaying the guest. KIP-20 is how the deposit and the settlement stay on the same lineage. KIP-21 is how the proof quotes the program's own lane. Maxim's name is on that KIP. The guest he wrote is the program the lane is for.

The demo's covenant is not an Argent program. It is those delegate-script bytes plus the settlement redeem script. Argent becomes relevant the day someone wants that redeem script reviewed as source instead of as opcodes. It is not relevant to the board rules. The board rules are the guest, because a multi-round account, a pending ring, a pot, and an exit proof are past what the script meter is for.

## How the guest and the paper meet, and where they miss

They meet on sovereignty of one program. The guest owns its resources. A turn that debits more than the balance fails inside the program (`checked_sub`), and the exec-mode storm measured that failure as zero successful thefts. The L1 covenant does not know what a tic-tac-toe line is. It knows the proof and the lineage. That is the based zk app Sutton described.

They miss on composition. Nothing in the guest reads an account that belongs to a different program. Deposit plus create in one carrier is one program paying itself. Topic 387's transaction, the one that is atomic across two owners, is not this match.

They miss on the failed write. Topic 410 wants a declared write that fails to still occupy its slot, copying the old value, so the caller pays for the scope it declared and does not surprise the next transaction. `commit_explicit` does the stricter thing: an occupied cell rejects the whole action before any write, and a cell from the waiting seat is stored and may land later, including on a board the caller has not seen yet.

That second miss is the one this crate measures, because it is entirely inside the guest and it is what a blind client does.

`play_turn` calls `commit_explicit` and then `drain_pending`. `commit_explicit` rejects when the cell is occupied, even if the seat only meant to precommit. Otherwise, if it is that seat's ply, the cell is the move. If it is not, the cell is pushed, up to four. `drain_pending` then pops the seat that is on move and keeps going across the round boundary. The comment in the file says the queues are ply-agnostic. `end_match_if_decided` clears them only when `match_outcome` has a winner. So:

- Best of three, creator to move, one X away from a line. Joiner submits an empty cell. It waits. Creator takes the line. The board clears, the match is 1-0 and still open, and the waiting cell is applied as the joiner's X on the fresh board. Round two has started inside the transaction that finished round one.
- One round, same queue, same winning cell. The match ends, the ring is cleared, the next board stays empty.

`land_batch` takes those same two cells as one declaration. The creator's winning cell is the ply. The joiner's cell is `Retry`. The next board stays empty on the best of three as well. `cargo run --example boards` prints both.

This is the storm's missing referee, shrunk to the function the guest already has. The campaign sent create, join, and cells 0, 3, 4, 6, 8, and it would not spend a transaction younger than 25 seconds. It could not see which attempt had landed, because the API was returning a document. A client in that state resubmits. A ring that treats a resubmit as a precommit of the next round will play a round nobody has looked at. A batch that lands one cell, and names the reason for every other cell, is something the client can show.

The 25-step gap is that orphan wait. It is not the guest's turn TTL, which is a DAA deadline and a forfeit. Both are clocks. They answer different questions. The TTL forfeits a seat who went silent. The gap refuses a spend the node will reject as an orphan. The batch rule uses the gap so a too-soon resubmit is red and the seat does not move.

Two holes at the same tip are recorded and not "fixed" here, because they are create-time and this crate starts from a playing match. `apply_create_game` rejects `rounds == 0`. `write_game` stores `rounds_total` and `stake` with no such check. `checked_sub(0)` succeeds, so a zero stake is a successful debit. Create in the JavaScript referee `blue-line-tictactoe` refuses both. The guest, as read, does not.

## Possibilities

In the order a developer can actually touch them.

1. Keep the guest as the rules and the covenant as the lock. The board, the seat score, and the clinch stay in `rules.rs`. The redeem script stays the thing a full node runs. Mixing them makes a script the meter will not hold, or a proof the node cannot check.

2. When the redeem script needs review, compile it from SilverScript and check the artifact against `delegate_script.rs`. Argent is the step after an expert is willing to own the generated `.sil`. It is not the step that makes the guest correct.

3. Treat a blind resubmit as a declared batch. One cell lands. A second copy of that cell is a second spend. Any other cell in the batch is a retry, including the cell that `drain_pending` would play on the next round. The tests in this crate are that rule next to a miniature of the current drain. They are not a patch on the guest checkout.

4. A quiet single match on the hosted lane, create, join, five moves, 25 seconds between spends, while the API is returning JSON. Record whether a second cell submitted in the same moment becomes the next round under the real `drain_pending`. This crate says what the pure functions do. It does not say what the hosted process did.

5. The yellow-paper transaction is a different project. It needs two programs, one declared read set, a stitched proof, and the L1 Sutton said is still missing. A tic-tac-toe match will not become that transaction by growing more rounds. KIP-21 is the lane half, and it is already Active. The account-level half is the open research.

6. The measurements that would move the public Testnet 10 verdict are still open, and this repo does not run them. A selected-chain rate above the packing ceilings. The mempool assert at the default cap of 1,000,000, rather than at the lowered cap where `100001 > 100000` fired. Pull 165 retested with a UTXO index under a flood, with the fee per game written down. A cause for the indexer acceptance clock that was still 25 Sep 2026 19:55:38Z on the 29 Sep cache miss. A fee window against miner coinbase, as a net. The round-5 storage-mass figure recomputed from a raw transaction.

## Sources

- research.kas.pa topics 247, 258, 293, 323, 347, 375, 387, 407, 410, 411, 494. Read into STP-KAS/kaspa-master-file at `34ecaf7`, file `RESEARCH-KAS-PA.md`. Topic 387 is the architecture. It is not a product.
- kaspanet/kips at `e4ae233`. KIP-16 zk precompile (Alexander Safstrom). KIP-17 covenants (Ori Newman). KIP-20 covenant ids (Michael Sutton). KIP-21 partitioned sequencing (Sutton, Maxim Biryukov, Hans Moog). All four Active.
- kaspanet/vprogs `f9b84a8` (master, 28 Jul 2026) and `07d486dd` (the settlement-watch tip, not a release). `zk/abi/src/delegate_script.rs`, `zk/abi/src/batch_processor/verifier.rs`.
- biryukovmaxim/vprog-tictactoe `098be674`. `guest/src/program/rules.rs`, `guest/src/program/action/game.rs`, `guest/src/program/resources/game.rs`.
- Michael Sutton, 26 Sep 2026, post 2103892665435578727. The demo is a standalone verifiable program that composes through L1. The yellow-paper design still needs L1 work.
- Max, 15 Sep 2026 (post 2099841318847729878), 18 Sep 2026 (2100806415510319354), 25 Sep 2026 (2103498627130003761). Data availability at the vprog layer, settlement back to L1, prove-and-settle as what you run today.
- kaspanet/silverscript tag v1.0.0 `3ed9733`. argent-lang/argent master `e76ee07`, no tags, not release-ready on the authors' own README.
- STP-KAS/tn10-vprogs-final-verdict, 26 Sep 2026. STP-KAS/kaspa-master-file `34ecaf7` for the pins and the 29 Sep log recount.
- Private STP-KAS/grok-bot-vprogs for the five-cell line and the 25-second wait. No seed and no address is in this repo.

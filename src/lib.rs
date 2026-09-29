//! One landed ply out of a declared batch.
//!
//! `guest_act` is the landing rule in
//! [biryukovmaxim/vprog-tictactoe](https://github.com/biryukovmaxim/vprog-tictactoe)
//! at tip `098be674`, function by function: `commit_explicit`, `drain_pending`,
//! `apply_move`, `close_round`, and `end_match_if_decided` in
//! `guest/src/program/action/game.rs`. A cell from the seat that is not on move
//! waits on that seat's ring of four. After the explicit cell, the rings drain,
//! and a waiting cell can become the first ply of the next round while the match
//! is still playing. When the match ends, the rings are cleared first.
//!
//! `land_batch` is the other rule. The caller declares every attempt. Exactly one
//! attempt becomes the ply. A cell that would open the next round is reported
//! and does not touch the board.
//!
//! `match_outcome` follows `guest/src/program/rules.rs` at that same tip.
//! Creator mark is X, which is mark byte 1, the value the campaign create passes.
//! This crate does not submit, does not prove, and does not link the guest.

use std::collections::VecDeque;

/// The eight lines, in the guest's index order.
pub const LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

/// Cells one seat may hold aside. `PENDING_CAP` in the guest.
pub const PENDING_CAP: usize = 4;

/// Stand-in for the campaign's 25-second orphan wait, in sequence units.
pub const ORPHAN_GAP: u64 = 25;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    X,
    O,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Seat {
    Creator,
    Joiner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Creator,
    Joiner,
    Draw,
}

/// Why an attempt in a batch did not become the ply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    BadSeq,
    BadCell,
    Immature,
    WrongSeat,
    Occupied,
    SecondSpend,
    Retry,
    MatchOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub seat: Seat,
    pub cell: u8,
    pub seq: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub seat: Seat,
    pub cell: u8,
    pub seq: u64,
    pub mark: Mark,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejected {
    pub seat: Seat,
    pub cell: u8,
    pub seq: u64,
    pub reason: Reason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Batch {
    pub game: Game,
    pub blue: Option<Landed>,
    pub red: Vec<Rejected>,
}

/// Match state. `pending` is only read by [`Game::guest_act`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub rounds: u8,
    pub gap: u64,
    pub wins: [u8; 2],
    pub draws: u8,
    pub board: [Option<Mark>; 9],
    pub last_seq: u64,
    pub over: Option<Outcome>,
    /// True only when the finish skipped leftover rounds.
    pub clinch: bool,
    pending: [VecDeque<u8>; 2],
}

impl Game {
    /// A playing match. Creator holds X. Rings empty.
    pub fn playing(rounds: u8, gap: u64) -> Result<Self, &'static str> {
        if rounds < 1 {
            return Err("rounds must be at least 1");
        }
        if gap < 1 {
            return Err("gap must be at least 1");
        }
        Ok(Self {
            rounds,
            gap,
            wins: [0, 0],
            draws: 0,
            board: [None; 9],
            last_seq: 0,
            over: None,
            clinch: false,
            pending: [VecDeque::new(), VecDeque::new()],
        })
    }

    /// Rounds already scored. The guest derives the round index the same way.
    pub fn round_index(&self) -> u8 {
        self.wins[0]
            .saturating_add(self.wins[1])
            .saturating_add(self.draws)
    }

    pub fn mark_for(&self, seat: Seat) -> Mark {
        let creator_is_x = self.round_index() % 2 == 0;
        match (seat, creator_is_x) {
            (Seat::Creator, true) | (Seat::Joiner, false) => Mark::X,
            _ => Mark::O,
        }
    }

    pub fn seat_to_move(&self) -> Seat {
        let marks = self.board.iter().filter(|cell| cell.is_some()).count();
        let opening = if marks % 2 == 0 { Mark::X } else { Mark::O };
        if self.mark_for(Seat::Creator) == opening {
            Seat::Creator
        } else {
            Seat::Joiner
        }
    }

    /// One guest action: the explicit cell, then the ring drain.
    ///
    /// An occupied cell rejects the action before any write, including a
    /// precommit. A full ring rejects. A finished match rejects.
    pub fn guest_act(&mut self, seat: Seat, cell: u8) -> Result<(), &'static str> {
        self.commit_explicit(seat, cell)?;
        self.drain_pending();
        Ok(())
    }

    fn commit_explicit(&mut self, seat: Seat, cell: u8) -> Result<(), &'static str> {
        if self.over.is_some() {
            return Err("turn: game is not playing");
        }
        if cell > 8 {
            return Err("turn: cell is out of range");
        }
        if self.board[cell as usize].is_some() {
            return Err("turn: cell is occupied");
        }
        if seat == self.seat_to_move() {
            self.apply_move(seat, cell);
        } else {
            let ring = &mut self.pending[seat_index(seat)];
            if ring.len() == PENDING_CAP {
                return Err("turn: pending queue is full");
            }
            ring.push_back(cell);
        }
        Ok(())
    }

    fn drain_pending(&mut self) {
        while self.over.is_none() {
            let seat = self.seat_to_move();
            let Some(cell) = self.pending[seat_index(seat)].pop_front() else {
                break;
            };
            if self.board[cell as usize].is_none() {
                self.apply_move(seat, cell);
            }
        }
    }

    fn apply_move(&mut self, seat: Seat, cell: u8) {
        let mark = self.mark_for(seat);
        self.board[cell as usize] = Some(mark);
        let won = line_winner(&self.board) == Some(mark);
        let full = !won && self.board.iter().all(|cell| cell.is_some());
        if !won && !full {
            return;
        }
        if won {
            self.wins[seat_index(seat)] = self.wins[seat_index(seat)].saturating_add(1);
        } else {
            self.draws = self.draws.saturating_add(1);
        }
        self.board = [None; 9];
        self.end_match_if_decided();
    }

    fn end_match_if_decided(&mut self) {
        let Some(outcome) = match_outcome(self.rounds, self.wins, self.draws) else {
            return;
        };
        let completed = self.wins[0] as u16 + self.wins[1] as u16 + self.draws as u16;
        let left = (self.rounds as u16).saturating_sub(completed);
        self.over = Some(outcome);
        self.clinch = left > 0;
        self.pending = [VecDeque::new(), VecDeque::new()];
    }
}

/// Same comparison as `rules::match_outcome`. Seat 0 is the creator.
pub fn match_outcome(rounds: u8, wins: [u8; 2], draws: u8) -> Option<Outcome> {
    let creator = wins[0] as u16;
    let joiner = wins[1] as u16;
    let completed = creator + joiner + draws as u16;
    let remaining = (rounds as u16).saturating_sub(completed);
    if creator > joiner + remaining {
        Some(Outcome::Creator)
    } else if joiner > creator + remaining {
        Some(Outcome::Joiner)
    } else if remaining == 0 {
        Some(match creator.cmp(&joiner) {
            std::cmp::Ordering::Greater => Outcome::Creator,
            std::cmp::Ordering::Less => Outcome::Joiner,
            std::cmp::Ordering::Equal => Outcome::Draw,
        })
    } else {
        None
    }
}

pub fn line_winner(board: &[Option<Mark>; 9]) -> Option<Mark> {
    LINES.iter().find_map(|[a, b, c]| {
        let mark = board[*a]?;
        (board[*b] == Some(mark) && board[*c] == Some(mark)).then_some(mark)
    })
}

/// Sort by sequence, then by input order. The first attempt that is the seat
/// to move, on an empty cell, and far enough past the previous ply, is the
/// one write. Later attempts are reported. Nothing is queued for a later round.
pub fn land_batch(game: &Game, attempts: &[Attempt]) -> Batch {
    let mut next = game.clone();
    let mut order: Vec<usize> = (0..attempts.len()).collect();
    order.sort_by(|&a, &b| {
        attempts[a]
            .seq
            .cmp(&attempts[b].seq)
            .then(a.cmp(&b))
    });
    let mut blue: Option<Landed> = None;
    let mut red = Vec::new();
    for index in order {
        let attempt = attempts[index];
        if let Some(reason) = classify(&next, attempt, blue.as_ref()) {
            red.push(Rejected {
                seat: attempt.seat,
                cell: attempt.cell,
                seq: attempt.seq,
                reason,
            });
            continue;
        }
        let mark = next.mark_for(attempt.seat);
        blue = Some(Landed {
            seat: attempt.seat,
            cell: attempt.cell,
            seq: attempt.seq,
            mark,
        });
        next.board[attempt.cell as usize] = Some(mark);
        next.last_seq = attempt.seq;
        let won = line_winner(&next.board) == Some(mark);
        let full = !won && next.board.iter().all(|cell| cell.is_some());
        if won || full {
            if won {
                let seat = seat_index(attempt.seat);
                next.wins[seat] = next.wins[seat].saturating_add(1);
            } else {
                next.draws = next.draws.saturating_add(1);
            }
            next.board = [None; 9];
            next.end_match_if_decided();
        }
    }
    Batch {
        game: next,
        blue,
        red,
    }
}

fn classify(game: &Game, attempt: Attempt, blue: Option<&Landed>) -> Option<Reason> {
    // Zero is the empty `last_seq`, not a ply a caller can name.
    if attempt.seq < 1 {
        return Some(Reason::BadSeq);
    }
    if let Some(blue) = blue {
        if attempt.seat == blue.seat && attempt.cell == blue.cell {
            return Some(Reason::SecondSpend);
        }
        return Some(Reason::Retry);
    }
    if game.over.is_some() {
        return Some(Reason::MatchOver);
    }
    if game.last_seq != 0 && attempt.seq < game.last_seq + game.gap {
        return Some(Reason::Immature);
    }
    if attempt.cell > 8 {
        return Some(Reason::BadCell);
    }
    if attempt.seat != game.seat_to_move() {
        return Some(Reason::WrongSeat);
    }
    if game.board[attempt.cell as usize].is_some() {
        return Some(Reason::Occupied);
    }
    None
}

fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::Creator => 0,
        Seat::Joiner => 1,
    }
}

/// Three text rows, `.` for an empty cell.
pub fn render(board: &[Option<Mark>; 9]) -> String {
    let cell = |index: usize| match board[index] {
        Some(Mark::X) => 'X',
        Some(Mark::O) => 'O',
        None => '.',
    };
    [0, 3, 6]
        .into_iter()
        .map(|start| format!("{} {} {}", cell(start), cell(start + 1), cell(start + 2)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Even round, creator to move, one X from a column win on cells 2, 4, 6.
/// Cell 8 is empty, so a queued joiner cell can name it.
pub fn column_ready(rounds: u8) -> Game {
    let mut game = Game::playing(rounds, ORPHAN_GAP).expect("rounds");
    game.guest_act(Seat::Creator, 2).expect("creator 2");
    game.guest_act(Seat::Joiner, 1).expect("joiner 1");
    game.guest_act(Seat::Creator, 4).expect("creator 4");
    game.guest_act(Seat::Joiner, 0).expect("joiner 0");
    game
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_refuses_zero_rounds() {
        assert_eq!(Game::playing(0, 1).unwrap_err(), "rounds must be at least 1");
    }

    #[test]
    fn clinch_math_matches_the_guest() {
        assert_eq!(match_outcome(5, [3, 0], 0), Some(Outcome::Creator));
        assert_eq!(match_outcome(5, [0, 3], 0), Some(Outcome::Joiner));
        assert_eq!(match_outcome(5, [2, 1], 0), None);
        assert_eq!(match_outcome(5, [2, 2], 0), None);
        assert_eq!(match_outcome(5, [3, 2], 0), Some(Outcome::Creator));
        assert_eq!(match_outcome(5, [2, 2], 1), Some(Outcome::Draw));
        assert_eq!(match_outcome(1, [1, 0], 0), Some(Outcome::Creator));
    }

    #[test]
    fn a_queued_cell_opens_the_next_round_while_the_match_continues() {
        let mut game = column_ready(3);
        assert_eq!(game.seat_to_move(), Seat::Creator);
        game.guest_act(Seat::Joiner, 8).unwrap();
        assert_eq!(game.board[8], None);
        game.guest_act(Seat::Creator, 6).unwrap();
        assert_eq!(game.wins, [1, 0]);
        assert_eq!(game.round_index(), 1);
        assert_eq!(game.over, None);
        assert_eq!(game.board[8], Some(Mark::X));
        assert_eq!(game.mark_for(Seat::Joiner), Mark::X);
    }

    #[test]
    fn the_same_queue_is_dropped_when_the_match_ends() {
        let mut game = column_ready(1);
        game.guest_act(Seat::Joiner, 8).unwrap();
        game.guest_act(Seat::Creator, 6).unwrap();
        assert_eq!(game.over, Some(Outcome::Creator));
        assert!(!game.clinch);
        assert_eq!(game.board, [None; 9]);
    }

    #[test]
    fn an_occupied_explicit_cell_rejects_before_any_write() {
        let mut game = column_ready(3);
        let before = game.clone();
        let err = game.guest_act(Seat::Creator, 2).unwrap_err();
        assert_eq!(err, "turn: cell is occupied");
        assert_eq!(game, before);
    }

    #[test]
    fn the_fifth_precommit_fills_the_ring() {
        let mut game = column_ready(3);
        for cell in [8, 7, 5, 3] {
            game.guest_act(Seat::Joiner, cell).unwrap();
        }
        assert_eq!(
            game.guest_act(Seat::Joiner, 6).unwrap_err(),
            "turn: pending queue is full"
        );
    }

    #[test]
    fn one_batch_does_not_open_the_next_round() {
        let game = column_ready(3);
        let step = land_batch(
            &game,
            &[
                Attempt { seat: Seat::Creator, cell: 6, seq: 40 },
                Attempt { seat: Seat::Joiner, cell: 8, seq: 41 },
            ],
        );
        assert_eq!(step.blue.as_ref().map(|landed| landed.cell), Some(6));
        assert_eq!(step.blue.as_ref().map(|landed| landed.mark), Some(Mark::X));
        assert_eq!(step.red.len(), 1);
        assert_eq!(step.red[0].reason, Reason::Retry);
        assert_eq!(step.game.board, [None; 9]);
        assert_eq!(step.game.wins, [1, 0]);
        assert_eq!(step.game.over, None);
        assert_eq!(step.game.round_index(), 1);
    }

    #[test]
    fn a_too_soon_attempt_and_a_second_spend_stay_off_the_board() {
        let mut game = Game::playing(1, ORPHAN_GAP).unwrap();
        let first = land_batch(
            &game,
            &[Attempt { seat: Seat::Creator, cell: 0, seq: 100 }],
        );
        game = first.game;
        let step = land_batch(
            &game,
            &[
                Attempt { seat: Seat::Joiner, cell: 1, seq: 110 },
                Attempt { seat: Seat::Joiner, cell: 1, seq: 125 },
                Attempt { seat: Seat::Joiner, cell: 1, seq: 126 },
            ],
        );
        assert_eq!(step.blue.as_ref().map(|landed| landed.cell), Some(1));
        assert_eq!(
            step.red.iter().map(|item| item.reason).collect::<Vec<_>>(),
            vec![Reason::Immature, Reason::SecondSpend]
        );
        assert_eq!(step.game.board[0], Some(Mark::X));
        assert_eq!(step.game.board[1], Some(Mark::O));
    }

    #[test]
    fn land_batch_leaves_its_input_alone() {
        let game = column_ready(3);
        let saved = game.clone();
        let _ = land_batch(
            &game,
            &[Attempt { seat: Seat::Creator, cell: 6, seq: 10 }],
        );
        assert_eq!(game, saved);
    }
}

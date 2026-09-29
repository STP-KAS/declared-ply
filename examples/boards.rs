//! Print the guest drain and the one-ply batch on the same position.
//!
//! The position is an even round, creator to move, one X away from the column
//! 2-4-6. Cell 8 is empty.

use ply_for_maxim::{column_ready, land_batch, render, Attempt, Mark, Seat};

fn main() {
    let base = column_ready(3);
    println!("Before the last ply. Creator to move. X is on 2 and 4, O is on 0 and 1.");
    println!("{}\n", render(&base.board));

    let mut guest = base.clone();
    guest.guest_act(Seat::Joiner, 8).expect("queue");
    guest.guest_act(Seat::Creator, 6).expect("win");
    println!("Guest rule. The joiner queued cell 8, then the creator played 6.");
    println!("The win cleared the board. The queued 8 opened the next round as X.");
    println!("{}", render(&guest.board));
    println!(
        "creator wins {}. round index {}. match still open: {}.\n",
        guest.wins[0],
        guest.round_index(),
        guest.over.is_none()
    );

    let ended = land_batch(
        &base,
        &[
            Attempt { seat: Seat::Creator, cell: 6, seq: 40 },
            Attempt { seat: Seat::Joiner, cell: 8, seq: 41 },
        ],
    );
    let blue = ended.blue.as_ref().expect("blue");
    println!("Batch rule. The same two cells are one declared ply.");
    println!("blue: {:?} {:?} on {}.", blue.seat, blue.mark, blue.cell);
    println!("red: {:?} cell {} {:?}.", ended.red[0].seat, ended.red[0].cell, ended.red[0].reason);
    println!("The next board stays empty.");
    println!("{}", render(&ended.game.board));
    println!(
        "creator wins {}. round index {}. match still open: {}.",
        ended.game.wins[0],
        ended.game.round_index(),
        ended.game.over.is_none()
    );

    let mut one = column_ready(1);
    one.guest_act(Seat::Joiner, 8).expect("queue");
    one.guest_act(Seat::Creator, 6).expect("win");
    println!("\nGuest rule, one round. The ring is cleared because the match ended.");
    println!("{}", render(&one.board));
    println!("over {:?} clinch {}.", one.over, one.clinch);

    assert_eq!(guest.board[8], Some(Mark::X));
    assert!(ended.game.board.iter().all(|cell| cell.is_none()));
    assert!(one.board.iter().all(|cell| cell.is_none()));
}

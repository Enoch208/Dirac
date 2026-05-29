use alloc::vec::Vec;

pub type PlayerId = [u8; 32];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entry {
    pub player: PlayerId,
    pub rating: i32,
}

pub fn update_leaderboard(
    board: &mut Vec<Entry>,
    player: PlayerId,
    rating: i32,
    capacity: usize,
) -> bool {
    let previous_champion = board.first().map(|entry| entry.player);

    match board.iter_mut().find(|entry| entry.player == player) {
        Some(entry) => entry.rating = rating,
        None => board.push(Entry { player, rating }),
    }

    board.sort_by(|a, b| b.rating.cmp(&a.rating).then(a.player.cmp(&b.player)));
    board.truncate(capacity);

    board.first().map(|entry| entry.player) != previous_champion
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u8) -> PlayerId {
        [n; 32]
    }

    #[test]
    fn first_player_becomes_champion() {
        let mut board = Vec::new();
        let champion_changed = update_leaderboard(&mut board, id(1), 1500, 8);
        assert!(champion_changed);
        assert_eq!(board[0].player, id(1));
    }

    #[test]
    fn overtaking_top_signals_new_champion() {
        let mut board = Vec::new();
        update_leaderboard(&mut board, id(1), 1500, 8);
        let changed = update_leaderboard(&mut board, id(2), 1600, 8);
        assert!(changed);
        assert_eq!(board[0].player, id(2));
    }

    #[test]
    fn updating_a_lower_player_keeps_champion() {
        let mut board = Vec::new();
        update_leaderboard(&mut board, id(1), 1600, 8);
        update_leaderboard(&mut board, id(2), 1400, 8);
        let changed = update_leaderboard(&mut board, id(2), 1550, 8);
        assert!(!changed);
        assert_eq!(board[0].player, id(1));
    }

    #[test]
    fn improving_existing_champion_is_not_a_new_champion() {
        let mut board = Vec::new();
        update_leaderboard(&mut board, id(1), 1500, 8);
        let changed = update_leaderboard(&mut board, id(1), 1900, 8);
        assert!(!changed);
        assert_eq!(board[0].player, id(1));
    }

    #[test]
    fn capacity_is_bounded_and_drops_lowest() {
        let mut board = Vec::new();
        for (player, rating) in [(1u8, 10), (2, 20), (3, 30), (4, 40)] {
            update_leaderboard(&mut board, id(player), rating, 3);
        }
        assert_eq!(board.len(), 3);
        assert_eq!(board[0].rating, 40);
        assert_eq!(board[2].rating, 20);
        assert!(!board.iter().any(|e| e.player == id(1)));
    }
}

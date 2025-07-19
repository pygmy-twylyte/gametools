//! # Dominos Module
//!
//! This module implements devices useful for working with / creating a game of Dominos.
//!
//! # Example
//! ```
//! use gametools::{BonePile, DominoHand, Train, GameResult};
//! # fn main() -> GameResult<()> {
//!
//! // set up a game using double-12 dominos. All trains must start
//! // with 12 for this round. We'll just create one open (public) train
//! // for simplicity.
//! let mut pile = BonePile::new(12);
//! let round_anchor = 12;
//! let mut public_train = Train::new("", true, round_anchor);
//!
//! // create a player's hand and their owned train, and draw 15 tiles from the pile
//! let player_name = "Zomby Woof";
//! let mut players_train = Train::new(&player_name, false, round_anchor);
//! let mut hand = DominoHand::new_with_draw(&player_name, 15, &mut pile)?;
//! println!("{hand}");
//!
//! // find the best (longest) initial play, and play it on the owned train
//! let mut longest_play = hand.find_longest_from(round_anchor);
//! hand.play_line(&longest_play, &mut players_train)?;
//! println!("After initial play -->");
//! println!("Remaining Hand: {hand}");
//! println!("Player's Train: {players_train}");
//!
//! # Ok(())
//! }
//! ```
use rand::prelude::SliceRandom;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::{GameError, GameResult};

/// The maximum number of pips allowed on each side of a domino.
pub const MAX_PIPS: u8 = 18;

/// A single domino tile.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Domino {
    left: u8,
    right: u8,
    id: usize,
}
impl Domino {
    /// Create a new domino.
    ///
    /// Typically called from Bonepile:new() when creating a new set of dominos,
    /// but can be used directly if desired. The id field is meant to hold a unique
    /// numeric id for the tile, making it easier to track when left and right can
    /// be flipped at any time.
    pub fn new(left: u8, right: u8, id: usize) -> Self {
        Self { left, right, id }
    }
    pub fn left(&self) -> u8 {
        self.left
    }
    pub fn right(&self) -> u8 {
        self.right
    }
    pub fn id(&self) -> usize {
        self.id
    }
    /// Returns a tuple containing (left, right, id) values for this domino.
    /// ```
    /// use gametools::Domino;
    /// let worst_tile = Domino::new(0, 0, 1);
    /// let (left, right, unique_id) = worst_tile.as_tuple();
    /// assert_eq!(left, worst_tile.left());
    /// assert_eq!(right, worst_tile.right());
    /// assert_eq!(unique_id, worst_tile.id());
    /// ```
    pub fn as_tuple(&self) -> (u8, u8, usize) {
        (self.left, self.right, self.id)
    }
    /// Returns a copy of this domino with left and right reversed, but same domino id#.
    /// ```
    /// # use gametools::Domino;
    /// let domino = Domino::new(1, 2, 4);
    /// let (left_orig, right_orig, id_orig) = domino.as_tuple();
    /// let flipped = domino.flipped();
    /// assert_eq!(flipped.right(), left_orig);
    /// assert_eq!(flipped.left(), right_orig);
    /// assert_eq!(flipped.id(), id_orig);
    /// ```
    pub fn flipped(&self) -> Self {
        Self {
            left: self.right,
            right: self.left,
            id: self.id,
        }
    }
    /// Returns the number of points this tile is worth, but assigning a special
    /// value to the 0-0 tile.
    /// ```
    /// # use gametools::Domino;
    /// let tile_0_0 = Domino::new(0,0,1);
    /// let tile_10_5 = Domino::new(10,5,2);
    ///
    /// assert_eq!(tile_0_0.points_with_zero_worth(50), 50);
    /// assert_eq!(tile_0_0.points_with_zero_worth(0), 0);
    /// assert_eq!(tile_10_5.points_with_zero_worth(50), 15);
    /// ```
    pub fn points_with_zero_worth(&self, value: u8) -> u8 {
        match self.left + self.right {
            0 => value,
            total => total,
        }
    }

    /// Returns the total number of pips on the tile.
    pub fn points(&self) -> u8 {
        self.left + self.right
    }
}
impl fmt::Display for Domino {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]", self.left, self.right)
    }
}

/// Pile of all of the dominos used for a game.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BonePile {
    tiles: Vec<Domino>,
}
impl BonePile {
    /// Create a new randomized set of dominos, specifying the maximum number of pips per side.
    ///
    /// This is capped at MAX_PIPS = 18 per side, the highest typically found in any domino set.
    pub fn new(most_pips: u8) -> Self {
        let mut tiles = Vec::<Domino>::new();
        let max = std::cmp::min(most_pips, MAX_PIPS);
        let mut did = 0;
        for left in 0..=max {
            for right in left..=max {
                tiles.push(Domino::new(left, right, did));
                did += 1;
            }
        }
        let mut rng = rand::rng();
        tiles.shuffle(&mut rng);
        Self { tiles }
    }
}
impl BonePile {
    /// Draw a single tile from the pile.
    ///
    /// Returns Some(Domino), or None if there are none left to draw.
    pub fn draw_tile(&mut self) -> Option<Domino> {
        self.tiles.pop()
    }
    /// Draws multiple tiles from the pile, usually used when creating a new hand.
    ///
    /// Returns `Some(Vec<Domino>)`, or `None` if there aren't enough dominos left to fill the request.
    pub fn draw_tiles(&mut self, count: usize) -> Option<Vec<Domino>> {
        if count > self.tiles.len() {
            None
        } else {
            Some(self.tiles.split_off(self.tiles.len() - count))
        }
    }
}

/// A train of dominos that have been played.
///
/// Player should be an empty string or other chosen token to indicate a public train,
/// or contain the name of the player if owned. For owned trains, 'open' refers to whether
/// other players are currently allowed to extend it. 'Head' is the starting value for the
/// round -- the initial value upon which the train must be built.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Train {
    player: String,
    open: bool,
    head: u8,
    tail: u8,
    tiles: Vec<Domino>,
}
impl fmt::Display for Train {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let open_or_closed = match self.open {
            true => "[O]-",
            false => "[X]-",
        };
        let head = format!("-({})", self.head);
        let mut output = open_or_closed.to_string();
        output.push_str(&self.player);
        output.push_str(&head);
        for tile in &self.tiles {
            output.push_str(&tile.to_string())
        }
        write!(f, "{}", output)
    }
}
impl Train {
    /// Create a new train.
    ///
    /// For a train that belongs to a player, it can be "closed" to plays from
    /// other players by setting the `open` field to `false`, then "opened" later
    /// by setting it to `true`.
    /// ```
    /// use gametools::Train;
    /// # let start_val = 0;
    /// let player_train = Train::new("JoePlayer#123", false, start_val);
    /// let community_train = Train::new("everyone", true, start_val);
    /// ```
    pub fn new(player: &str, open: bool, start: u8) -> Self {
        Self {
            player: player.to_owned(),
            open,
            head: start,
            tail: start,
            tiles: Vec::<Domino>::new(),
        }
    }
    /// Attempt to play a tile on the train.
    ///
    /// Returns Err(GameError) if it isn't a valid play or if the train
    /// is closed and doesn't belong to the calling player.
    pub fn play(&mut self, tile: Domino, player: &str) -> GameResult<()> {
        if !self.open && self.player != player {
            return Err(GameError::TrainClosed);
        }
        let new_tile = match tile {
            _ if tile.left == self.tail => tile,
            _ if tile.right == self.tail => tile.flipped(),
            _ => return Err(GameError::TileUnconnected),
        };
        self.tail = new_tile.right;
        self.tiles.push(new_tile);
        Ok(())
    }
}

/// A player's hand of dominos.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct DominoHand {
    player: String,
    tiles: Vec<Domino>,
}
impl fmt::Display for DominoHand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!("{}->", self.player);
        for tile in &self.tiles {
            output.push_str(&tile.to_string());
        }
        write!(f, "{}", output)
    }
}
impl DominoHand {
    /// Create a new empty hand.
    pub fn new(player: &str) -> Self {
        Self {
            player: player.to_owned(),
            tiles: Vec::<Domino>::new(),
        }
    }
    /// Create a new hand and draw tiles for it.
    ///
    /// This returns either Ok(DominoHand), or Err(GameError::InsufficientTiles).
    pub fn new_with_draw(player: &str, count: usize, pile: &mut BonePile) -> GameResult<Self> {
        if let Some(starting_tiles) = pile.draw_tiles(count) {
            Ok(Self {
                player: player.to_owned(),
                tiles: starting_tiles,
            })
        } else {
            Err(GameError::InsufficientTiles)
        }
    }

    /// Build the longest possible sequence of dominos from this hand, starting with the
    /// specified number.
    ///
    /// This is considered an NP-hard problem. The function uses a graph-based, depth first
    /// search with pruning and backtracking to find the optimal solution. For a typical starting
    /// hand of 15 tiles, execution takes around 200-300 ms on a modern processor (unoptimized + debug)...
    /// but it increases exponentially. A few runs of 25 tiles took anywhere from 11 sec to 3 min,
    /// and I didn't wait long enough for 30 tiles to finish.
    pub fn find_longest_from(&self, head: u8) -> Vec<usize> {
        // * build a graph - #pips are nodes, and dominos that connect them are edges
        // * modeled with a HashMap (key = #pips, val = list of domino ids that can connect to it)
        let mut graph = HashMap::<u8, Vec<(u8, usize)>>::new();
        for tile in &self.tiles {
            // each tile added twice since it can be used with left and right flipped at will
            graph
                .entry(tile.left)
                .or_default()
                .push((tile.right, tile.id));
            graph
                .entry(tile.right)
                .or_default()
                .push((tile.left, tile.id));
        }

        // initialize and start depth-first search
        let mut best_line = Vec::<usize>::new();
        let mut used = HashSet::<usize>::new(); // keeps track of tiles/edges already used
        let mut working_line = Vec::<usize>::new();
        Self::depth_first_search(&graph, head, &mut best_line, &mut used, &mut working_line);
        best_line
    }
    fn depth_first_search(
        graph: &HashMap<u8, Vec<(u8, usize)>>,
        head: u8,
        best: &mut Vec<usize>,
        used: &mut HashSet<usize>,
        working: &mut Vec<usize>,
    ) {
        let mut extended = false;

        for &(pips, domino_id) in graph.get(&head).unwrap_or(&vec![]) {
            if !used.contains(&domino_id) {
                used.insert(domino_id);
                working.push(domino_id);

                Self::depth_first_search(graph, pips, best, used, working);

                working.pop();
                used.remove(&domino_id);
                extended = true;
            }
        }

        if !extended && working.len() > best.len() {
            *best = working.clone();
        }
    }
    /// Takes a sequence of domino ids and attempt to play them on a train.
    ///
    /// _PANIC_ : if you pass a domino_id that doesn't exist in this hand
    ///
    /// The will return with an error if a tile doesn't match the one before it,
    /// or if this player doesn't have permission to use that train.
    pub fn play_line(&mut self, id_sequence: &Vec<usize>, train: &mut Train) -> GameResult<()> {
        for domino_id in id_sequence {
            let pos = self
                .tiles
                .iter()
                .position(|&t| t.id == *domino_id)
                .expect("specified domino id not found in play_line");
            let tile = self.tiles.swap_remove(pos);
            train.play(tile, &self.player)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod domino_tests {
    use crate::*;

    #[test]
    fn test_find_longest_from_returns_expected_ids() {
        use crate::{Domino, DominoHand};

        // Define a hand manually with known longest path
        let hand = vec![
            Domino::new(1, 2, 0),
            Domino::new(2, 3, 1),
            Domino::new(3, 4, 2),
            Domino::new(4, 1, 3), // This closes a loop
            Domino::new(0, 1, 4), // This adds an extension off 1
        ];

        let mut dom_hand = DominoHand::new("TestPlayer");
        dom_hand.tiles = hand;

        // Starting from pip 1
        let result = dom_hand.find_longest_from(1);

        // All 5 tiles are connectable in a valid path
        assert_eq!(result.len(), 5);

        // Since multiple valid orderings are possible, just check tile IDs
        let expected: Vec<usize> = vec![0, 1, 2, 3, 4];
        for id in expected {
            assert!(result.contains(&id));
        }
    }

    #[test]
    fn dominohand_new_works() {
        let dh = DominoHand::new("Zappa");
        assert_eq!(dh.tiles.len(), 0);
        assert_eq!(dh.player, "Zappa");
    }

    #[test]
    fn dominohand_new_with_draw_works() -> GameResult<()> {
        let mut bp = BonePile::new(12); // full 12-set = 169 tiles
        let dh = DominoHand::new_with_draw("Peart", 15, &mut bp)?;
        assert_eq!(dh.tiles.len(), 15);
        assert_eq!(bp.tiles.len(), 91 - 15);

        assert!(DominoHand::new_with_draw("who", 1000, &mut bp).is_err());
        Ok(())
    }

    #[test]
    fn bonepile_draw_tile_works() {
        let mut pile = BonePile::new(12);
        let _ = pile.draw_tile();
        assert_eq!(pile.tiles.len(), 90);
    }

    #[test]
    fn bonepile_draw_tiles_works() {
        let mut pile = BonePile::new(12);
        if let Some(some_tiles) = pile.draw_tiles(15) {
            assert_eq!(pile.tiles.len(), 91 - 15);
            assert_eq!(some_tiles.len(), 15);
        }

        let way_too_many = pile.draw_tiles(1000);
        assert!(way_too_many.is_none());
    }

    #[test]
    fn create_domino_works() {
        let tile = Domino::new(0, 1, 0);
        assert_eq!(tile.left(), 0);
        assert_eq!(tile.right(), 1);
    }

    #[test]
    fn domino_display_is_correct() {
        let tile = Domino::new(0, 0, 0);
        let another_tile = Domino::new(5, 5, 5);
        assert_eq!(tile.to_string(), "[0:0]");
        assert_eq!(another_tile.to_string(), "[5:5]");
    }

    #[test]
    fn flip_domino_works() {
        let tile = Domino::new(1, 2, 101);
        let flipped = tile.flipped();
        assert_eq!(flipped.id(), tile.id());
        assert_eq!(flipped.right(), tile.left());
        assert_eq!(flipped.left(), tile.right());
    }

    #[test]
    fn domino_as_tuple_is_correct() {
        let tile = Domino::new(1, 2, 3);
        assert_eq!(tile.as_tuple(), (1, 2, 3));
    }

    #[test]
    fn train_new_works() {
        let train = Train::new("zappa", false, 12);
        assert_eq!(train.player, "zappa");
        assert_eq!(train.open, false);
        assert_eq!(train.head, 12);
    }

    #[test]
    fn domino_points_is_correct() {
        let tile = Domino::new(0, 0, 0);
        assert_eq!(tile.points(), 0);

        let tile = Domino::new(12, 9, 1);
        assert_eq!(tile.points(), 21);
    }

    #[test]
    fn domino_points_with_zero_worth_is_correct() {
        let double_zero = Domino::new(0, 0, 0);
        let double_nine = Domino::new(9, 9, 1);
        assert_eq!(double_zero.points_with_zero_worth(0), 0);
        assert_eq!(double_nine.points_with_zero_worth(0), 18);
        assert_eq!(double_zero.points_with_zero_worth(50), 50);
        assert_eq!(double_nine.points_with_zero_worth(50), 18);
    }

    #[test]
    fn create_bonepile_works() {
        let six_pile = BonePile::new(6);
        let twelve_pile = BonePile::new(12);
        let over_max = BonePile::new(50); // should still only go up to MAX_PIPS
        assert_eq!(six_pile.tiles.len(), 28);
        assert_eq!(twelve_pile.tiles.len(), 91);
        assert_eq!(over_max.tiles.len(), 190); // number of tiles in a double-18 (MAX_PIPS) set
    }

    #[test]
    fn train_display_is_correct() {
        let private = Train::new("moon", false, 12);
        let mut public = Train::new("open", true, 12);
        let tile_12_1 = Domino::new(12, 1, 0);
        public.play(tile_12_1, "moon").unwrap();

        assert_eq!(private.to_string(), "[X]-moon-(12)");
        assert_eq!(public.to_string(), "[O]-open-(12)[12:1]");
    }

    #[test]
    fn train_play_works() {
        let mut public = Train::new("open", true, 12);
        let mut private = Train::new("bonzo", false, 12);
        let d12_1 = Domino::new(12, 1, 0);
        let d2_12 = Domino::new(2, 12, 1); // this one will have to flip
        let d5_6 = Domino::new(5, 6, 2); // this one won't fit at all

        assert!(private.play(d12_1, "percy").is_err()); // closed private train, wrong player
        assert!(private.play(d12_1, "bonzo").is_ok()); // closed private train, player owns it
        assert!(private.tail == 1);
        assert!(private.tiles.len() == 1);

        assert!(public.play(d2_12, "anyone").is_ok());
        assert!(public.tail == 2); // this tile had to flip (2_12 -> 12_2) to play on the train
        assert!(public.tiles.len() == 1);

        assert!(public.play(d5_6, "anyone").is_err()); // wrong #s to play on tail of this train
    }

    #[test]
    fn hand_display_works() {
        let mut hand = DominoHand::new("me");
        hand.tiles.push(Domino {
            left: 1,
            right: 1,
            id: 1,
        });
        assert_eq!(hand.to_string(), "me->[1:1]");
    }

    #[test]
    #[should_panic]
    fn hand_play_line_panics_on_bad_id() {
        // create a hand with 3 sequential dominos and an open/community train
        let mut hand = DominoHand::new("test");
        hand.tiles = vec![
            Domino::new(1, 2, 0),
            Domino::new(2, 3, 1),
            Domino::new(3, 4, 2),
        ];
        let mut train = Train::new("open", true, 1);

        let bad_sequence = vec![9, 8, 9, 8]; // these domino ids aren't in the hand
        let _result = hand.play_line(&bad_sequence, &mut train); // should panic!
    }

    #[test]
    fn hand_play_line_works() {
        // create a hand with 3 sequential dominos and an open/community train
        let mut hand = DominoHand::new("test");
        hand.tiles = vec![
            Domino::new(1, 2, 0),
            Domino::new(2, 3, 1),
            Domino::new(3, 4, 2),
        ];
        let mut train = Train::new("open", true, 1);

        let valid_sequence = vec![0, 1, 2]; // these domino ids should play in sequence
        let _result = hand.play_line(&valid_sequence, &mut train);
        assert!(hand.tiles.is_empty());
        assert_eq!(train.tiles.len(), 3);
        assert_eq!(train.tail, 4); // last tile should be [3:4]
    }
}

//! # Dominos
//!
//! This module implements devices useful for working with / creating a game of Dominos.
use rand::prelude::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::{GameError, GameResult};

pub const MAX_PIPS: u8 = 12;

/// A single domino tile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Domino {
    left: u8,
    right: u8,
    did: usize,
}
impl Domino {
    pub fn new(left: u8, right: u8, did: usize) -> Self {
        Self { left, right, did }
    }
    pub fn left(&self) -> u8 {
        self.left
    }
    pub fn right(&self) -> u8 {
        self.right
    }
    pub fn as_tuple(&self) -> (u8, u8, usize) {
        (self.left, self.right, self.did)
    }
    /// Returns a copy of this domino with left and right reversed, but same domino id#.
    pub fn flipped(&self) -> Self {
        Self {
            left: self.right,
            right: self.left,
            did: self.did,
        }
    }
    /// Returns the number of points this tile is worth. 0-0 is worth 50.
    pub fn points(&self) -> u8 {
        match self.left + self.right {
            0 => 50,
            total => total,
        }
    }
}
impl fmt::Display for Domino {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]", self.left, self.right)
    }
}

/// Pile of all of the dominos used for a game.
#[derive(Debug, Clone)]
pub struct BonePile {
    tiles: Vec<Domino>,
}
impl BonePile {
    /// Create a new randomized set of dominos, specifying the maximum number of pips per side.
    pub fn new(most_pips: u8) -> Self {
        let mut tiles = Vec::<Domino>::new();
        let max = std::cmp::min(most_pips, MAX_PIPS);
        let mut did = 0;
        for left in 0..=max {
            for right in 0..=max {
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
    pub fn draw_tile(&mut self) -> Option<Domino> {
        self.tiles.pop()
    }
    /// Draws multiple tiles from the pile, usually only used when creating a new hand.
    pub fn draw_tiles(&mut self, count: usize) -> Option<Vec<Domino>> {
        if count > self.tiles.len() {
            return None;
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
/// round, upon which the train must be built.
#[derive(Debug, Clone)]
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
        let mut output = format!("{open_or_closed}");
        output.push_str(&self.player);
        output.push_str(&head);
        for tile in &self.tiles {
            output.push_str(&tile.to_string())
        }
        write!(f,"{}", output)
    }
}
impl Train {
    pub fn new(player: &str, open: bool, start: u8) -> Self {
        Self {
            player: player.to_owned(),
            open,
            head: start,
            tail: start,
            tiles: Vec::<Domino>::new(),
        }
    }
    pub fn play(&mut self, tile: Domino) {
        // flip the domino before placement if needed
        let new_tile = match self.tail == tile.left {
            true => tile,
            false => tile.flipped(),
        };
        self.tail = new_tile.right;
        self.tiles.push(new_tile);
    }
}

/// A player's hand of dominos.
#[derive(Debug, Clone)]
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
        // don't initialize the hand with tiles here, because the possibility of insufficient
        // tiles in the pile would mean that new() could fail, and we don't want to have to
        // return a GameResult<> from the constructor. That's somewhat unexpected behavior, so
        // we'll have a separate one-step constructor for optional use.
        Self {
            player: player.to_owned(),
            tiles: Vec::<Domino>::new(),
        }
    }
    /// Create and draw tiles for a new hand.
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

    /// Get a list of domino id's in the hand for the longest possible train starting with a given number.
    pub fn find_longest_from(&self, head: u8) -> Vec<usize> {
        // * build a graph - pips are nodes, and dominos that connect them are edges
        // * we model this with a HashMap (key = # of pips, val = list of dominos that can connect to it)
        // the types below make the code below a little easier to read / understand... for me at least :)

        let mut graph = HashMap::<u8, Vec<(u8, usize)>>::new();
        for tile in &self.tiles {
            graph
                .entry(tile.left)
                .or_default()
                .push((tile.right, tile.did));
            graph
                .entry(tile.right)
                .or_default()
                .push((tile.left, tile.did));
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

        for &(pips, did) in graph.get(&head).unwrap_or(&vec![]) {
            if !used.contains(&did) {
                used.insert(did);
                working.push(did);

                Self::depth_first_search(&graph, pips, best, used, working);

                working.pop();
                used.remove(&did);
                extended = true;
            }
        }

        if !extended && working.len() > best.len() {
            *best = working.clone();
        }
    }
    /// Takes results from find_longest_from() and plays that line on a train
    pub fn play_line(&mut self, line_dids: &Vec<usize>, train: &mut Train) {
        for domino_id in line_dids {
            let pos = self
                .tiles
                .iter()
                .position(|&t| t.did == *domino_id)
                .expect("specified domino id not found in play_line");
            let tile = self.tiles.swap_remove(pos);
            train.play(tile);
        }
    }
}

#[cfg(test)]
mod domino_tests {
    use crate::*;

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
        assert_eq!(bp.tiles.len(), 169 - 15);
        Ok(())
    }

    #[test]
    fn bonepile_draw_tile_works() {
        let mut pile = BonePile::new(12);
        let _ = pile.draw_tile();
        assert_eq!(pile.tiles.len(), 168);
    }

    #[test]
    fn bonepile_draw_tiles_works() {
        let mut pile = BonePile::new(12);
        let _ = pile.draw_tiles(15);
        assert_eq!(pile.tiles.len(), 154);
    }

    #[test]
    fn create_domino_works() {
        let tile = Domino::new(0, 1, 0);
        assert_eq!(tile.left(), 0);
        assert_eq!(tile.right(), 1);
    }

    #[test]
    fn flip_domino_works() {
        let tile = Domino::new(1, 2, 101);
        assert_eq!(tile.flipped(), Domino::new(2, 1, 101));
    }

    #[test]
    fn domino_points_is_correct() {
        let tile = Domino::new(0, 0, 0);
        assert_eq!(tile.points(), 50);

        let tile = Domino::new(12, 9, 1);
        assert_eq!(tile.points(), 21);
    }

    #[test]
    fn create_bonepile_works() {
        let six_pile = BonePile::new(6);
        let twelve_pile = BonePile::new(12);
        let over_max = BonePile::new(50); // should still only go up to MAX_PIPS
        assert_eq!(six_pile.tiles.len(), 7 * 7);
        assert_eq!(twelve_pile.tiles.len(), 13 * 13);
        assert_eq!(
            over_max.tiles.len(),
            ((MAX_PIPS + 1) * (MAX_PIPS + 1)) as usize
        );
    }
}

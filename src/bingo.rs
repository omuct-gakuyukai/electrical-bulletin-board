use bevy::prelude::*;
use rand::seq::SliceRandom;

#[derive(Resource, Default)]
pub struct BingoState {
    pub numbers: Vec<u8>,
    pub index: usize,
}

impl BingoState {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut n: Vec<u8> = (1..=75).collect();
        n.shuffle(&mut rng);

        Self {
            numbers: n,
            index: 0,
        }
    }

    pub fn next(&mut self) -> Option<u8> {
        if self.index < self.numbers.len() {
            let num = self.numbers[self.index];
            self.index += 1;
            Some(num)
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<u8> {
        if self.index < self.numbers.len() {
            let num = self.numbers[self.index];
            Some(num)
        } else {
            None
        }
    }
}

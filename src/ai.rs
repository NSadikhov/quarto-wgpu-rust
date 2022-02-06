use crate::game::{Coordinate, Game, Piece};
use oorandom;
// use crate::Game;

// ! Improve AI ! (implement MinMax decision-making algorithm)

// pub struct Piece {
//     color: &'static str,
//     shape: &'static str,
//     height: &'static str,
//     state: &'static str,
// }

const MOVES_PLACES_1: [Coordinate; 4] = [
    Coordinate { row: 3, col: 0 },
    Coordinate { row: 1, col: 0 },
    Coordinate { row: 0, col: 0 },
    Coordinate { row: 3, col: 3 },
];

const MOVES_PIECES_1: [&str; 3] = [
    "Light_Round_Short_Solid",
    "Dark_Round_Short_Solid",
    "Light_Round_Short_Hollow",
];

const MOVES_PLACES_2: [Coordinate; 4] = [
    Coordinate { row: 0, col: 1 },
    Coordinate { row: 2, col: 3 },
    Coordinate { row: 2, col: 2 },
    Coordinate { row: 3, col: 3 },
];

const MOVES_PIECES_2: [&str; 3] = [
    "Light_Round_Tall_Hollow",
    "Light_Square_Short_Solid",
    "Dark_Square_Short_Hollow",
];

#[derive(Debug, Clone)]
pub struct AI {
    move_places_counter: usize,
    move_pieces_counter: usize,
    seed: u64,
    level: usize,
}

impl AI {
    pub fn init(level: usize) -> Self {
        let move_places_counter = 0;
        let move_pieces_counter = 0;
        let seed = 4;
        Self {
            move_places_counter,
            move_pieces_counter,
            seed,
            level,
        }
    }

    pub fn select_piece(&mut self, available_pieces: &mut Vec<Piece>, selected_piece: &mut Piece) {
        let mut MOVES_PIECES = vec![];

        if self.level == 1 {
            MOVES_PIECES = MOVES_PIECES_1.to_vec();
        } else if self.level == 2 {
            MOVES_PIECES = MOVES_PIECES_2.to_vec();
        }

        if self.move_pieces_counter < MOVES_PIECES.len()
            && available_pieces
                .iter()
                .any(|each| each.name == MOVES_PIECES[self.move_pieces_counter])
        {
            let index = available_pieces
                .iter()
                .position(|each| each.name == MOVES_PIECES[self.move_pieces_counter])
                .unwrap();

            *selected_piece = available_pieces[index];

            self.move_pieces_counter += 1;
        } else {
            let size = available_pieces.len();

            self.seed += 1;
            let mut rng = oorandom::Rand32::new(self.seed);
            let index = if size > 1 {
                rng.rand_range(0..size as u32 - 1)
            } else {
                0
            };

            // println!("select_place random index: {}", index);
            // println!("select_place len: {}", size);

            *selected_piece = available_pieces[index as usize];
        }
    }

    pub fn select_place(
        &mut self,
        board: &mut Vec<(Coordinate, Option<Piece>)>,
        selected_piece: Piece,
        selected_coor: &mut Option<Coordinate>,
    ) {
        let mut MOVES_PLACES = vec![];

        if self.level == 1 {
            MOVES_PLACES = MOVES_PLACES_1.to_vec();
        } else if self.level == 2 {
            MOVES_PLACES = MOVES_PLACES_2.to_vec();
        }

        println!("{}", self.level);

        if self.move_places_counter < MOVES_PLACES.len()
            && board.iter().any(|each| {
                each.0.row == MOVES_PLACES[self.move_places_counter].row
                    && each.0.col == MOVES_PLACES[self.move_places_counter].col
                    && each.1.is_none()
            })
        {
            let index = board
                .iter()
                .position(|each| {
                    each.0.row == MOVES_PLACES[self.move_places_counter].row
                        && each.0.col == MOVES_PLACES[self.move_places_counter].col
                        && each.1.is_none()
                })
                .unwrap();

            *selected_coor = Some(board[index].0);
            board[index].1 = Some(selected_piece);

            self.move_places_counter += 1;
        } else {
            let mut board_filtered = board
                .iter_mut()
                .filter(|each| each.1.is_none())
                .collect::<Vec<_>>();

            let size = board_filtered.len();

            self.seed += 1;
            let mut rng = oorandom::Rand32::new(self.seed);
            let index = if size > 1 {
                rng.rand_range(0..size as u32 - 1)
            } else {
                0
            };
            // println!("select_place random index: {}", index);
            // println!("select_place size: {}", size);

            *selected_coor = Some(board_filtered[index as usize].0);
            board_filtered[index as usize].1 = Some(selected_piece);
        }
    }
}

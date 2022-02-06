use std::collections::HashMap;

use cgmath::prelude::*;
use winit::event::{
    ElementState, KeyboardInput,
    MouseScrollDelta::{self, LineDelta, PixelDelta},
    TouchPhase, VirtualKeyCode, WindowEvent,
};

use crate::{ai::AI, instance, model::Model};

// pub const GAME_PIECES_NAMES: [&str; 16] = [
//     "Light_Round_Tall_Hollow",
//     "Light_Round_Tall_Solid",
//     "Light_Square_Tall_Hollow",
//     "Light_Square_Tall_Solid",
//     "Light_Square_Short_Hollow",
//     "Light_Square_Short_Solid",
//     "Light_Round_Short_Hollow",
//     "Light_Round_Short_Solid",
//     "Dark_Round_Short_Hollow",
//     "Dark_Round_Short_Solid",
//     "Dark_Square_Short_Hollow",
//     "Dark_Square_Short_Solid",
//     "Dark_Square_Tall_Hollow",
//     "Dark_Square_Tall_Solid",
//     "Dark_Round_Tall_Hollow",
//     "Dark_Round_Tall_Solid",
// ];

pub const GAME_PIECES_NAMES: [(&str, [f32; 3], [f32; 3]); 16] = [
    (
        "Light_Round_Tall_Solid",
        [0.0, 0.1, 0.0],
        [1.15, -0.1, -7.375],
    ),
    (
        "Light_Round_Tall_Hollow",
        [0.1, 0.1, -1.0],
        [1.15, -0.1, -6.375],
    ),
    //
    (
        "Light_Square_Tall_Solid",
        [1.15, 0.1, 0.0],
        [0.0, -0.1, -7.3],
    ),
    (
        "Light_Square_Tall_Hollow",
        [1.15, 0.1, -1.0],
        [0.0, -0.1, -6.3],
    ),
    //
    (
        "Light_Square_Short_Solid",
        [2.2, -0.4, 0.0],
        [-1.0, -0.1, -7.2],
    ),
    (
        "Light_Square_Short_Hollow",
        [2.2, -0.4, -1.0],
        [-1.05, -0.1, -6.325],
    ),
    //
    (
        "Light_Round_Short_Solid",
        [3.1, -0.4, 0.0],
        [-1.9, -0.1, -7.25],
    ),
    (
        "Light_Round_Short_Hollow",
        [3.1, -0.4, -1.0],
        [-1.9, -0.1, -6.3],
    ),
    //
    (
        "Dark_Round_Short_Solid",
        [3.8, -0.4, 0.0],
        [-2.75, -0.1, -7.2],
    ),
    (
        "Dark_Round_Short_Hollow",
        [3.8, -0.4, -1.0],
        [-2.75, -0.1, -6.3],
    ),
    //
    (
        "Dark_Square_Short_Solid",
        [4.6, -0.4, 0.0],
        [-3.5, -0.1, -7.275],
    ),
    (
        "Dark_Square_Short_Hollow",
        [4.6, -0.4, -1.0],
        [-3.5, -0.1, -6.3],
    ),
    //
    (
        "Dark_Square_Tall_Solid",
        [5.4, 0.1, 0.0],
        [-4.3, -0.1, -7.275],
    ),
    (
        "Dark_Square_Tall_Hollow",
        [5.5, 0.1, -1.0],
        [-4.4, -0.1, -6.3],
    ),
    //
    (
        "Dark_Round_Tall_Solid",
        [6.3, 0.1, 0.0],
        [-5.2, -0.1, -7.35],
    ),
    (
        "Dark_Round_Tall_Hollow",
        [6.4, 0.1, -1.0],
        [-5.3, -0.1, -6.4],
    ),
];

pub const BOARD_COORDS_ROWS_NUM: usize = 4;
pub const BOARD_COORDS_COLUMNS_NUM: usize = 4;
pub const BOARD_COORDS_NUM: usize = BOARD_COORDS_ROWS_NUM * BOARD_COORDS_COLUMNS_NUM;

pub fn get_board_coords() -> [(Coordinate, [f32; 3]); BOARD_COORDS_NUM] {
    let mut board_coords: [(Coordinate, [f32; 3]); BOARD_COORDS_NUM] =
        [(Coordinate { row: 0, col: 0 }, [0.0, 0.0, 0.0]); BOARD_COORDS_NUM];

    let mut i = 0;
    for row in 0..BOARD_COORDS_ROWS_NUM as i8 {
        for col in 0..BOARD_COORDS_COLUMNS_NUM as i8 {
            let space_x: f32 = if col == 0 { 0.0 } else { col as f32 * 0.3 };
            let space_z: f32 = if row == 0 { 0.0 } else { row as f32 * 0.3 };

            board_coords[i] = (
                Coordinate { row, col },
                [col as f32 + space_x, 0.0, row as f32 + space_z],
            );
            i += 1;
        }
    }

    board_coords
}

#[derive(Debug, Copy, Clone)]
pub enum Turn {
    Player,
    Opponent,
}

#[derive(Debug, Copy, Clone)]
pub struct Coordinate {
    pub row: i8,
    pub col: i8,
}

#[derive(Debug, Copy, Clone)]
pub struct Piece {
    pub name: &'static str,
    pub arrow_point: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Game {
    pub turn: Turn,
    pub selected_piece: Piece,
    pub selected_coor: Option<Coordinate>,
    pub available_pieces: Vec<Piece>,
    pub board: Vec<(Coordinate, Option<Piece>)>,
    pub piece_played: bool,
    pub ai: AI,
    pub ended: bool,
}

impl Game {
    pub fn init(level: usize) -> Self {
        let available_pieces = GAME_PIECES_NAMES
            .map(|p| Piece {
                name: p.0,
                arrow_point: p.1,
            })
            .to_vec();

        let mut board = Vec::<(Coordinate, Option<Piece>)>::new();
        for row in 0..4 {
            for col in 0..4 {
                board.push((Coordinate { row, col }, None));
            }
        }

        let ai = AI::init(level);

        Self {
            turn: Turn::Player,
            selected_piece: available_pieces[0],
            selected_coor: None,
            available_pieces,
            board,
            piece_played: false,
            ai,
            ended: false,
        }
    }

    pub fn reset(&mut self, level: usize) {
        *self = Game::init(level);
    }

    pub fn has_same_feature(&self, board_filtered: &Vec<&(Coordinate, Option<Piece>)>) -> bool {
        let mut same_color = false;
        let mut same_shape = false;
        let mut same_height = false;
        let mut same_state = false;

        let piece_0: Vec<_> = board_filtered[0].1.unwrap().name.split("_").collect();
        let piece_1: Vec<_> = board_filtered[1].1.unwrap().name.split("_").collect();
        let piece_2: Vec<_> = board_filtered[2].1.unwrap().name.split("_").collect();
        let piece_3: Vec<_> = board_filtered[3].1.unwrap().name.split("_").collect();

        if piece_0[0] == piece_1[0] && piece_1[0] == piece_2[0] && piece_2[0] == piece_3[0] {
            same_color = true;
        } else if piece_0[1] == piece_1[1] && piece_1[1] == piece_2[1] && piece_2[1] == piece_3[1] {
            same_shape = true;
        } else if piece_0[2] == piece_1[2] && piece_1[2] == piece_2[2] && piece_2[2] == piece_3[2] {
            same_height = true;
        } else if piece_0[3] == piece_1[3] && piece_1[3] == piece_2[3] && piece_2[3] == piece_3[3] {
            same_state = true;
        }

        same_color || same_shape || same_height || same_state
    }

    pub fn check_row(&self, n: i8) -> bool {
        let board_filtered = self
            .board
            .iter()
            .filter(|each| each.0.row == n && each.1.is_some())
            .collect::<Vec<_>>();

        if board_filtered.len() < 4 {
            return false;
        }

        self.has_same_feature(&board_filtered)
    }

    pub fn check_col(&self, n: i8) -> bool {
        let board_filtered = self
            .board
            .iter()
            .filter(|each| each.0.col == n && each.1.is_some())
            .collect::<Vec<_>>();

        if board_filtered.len() < 4 {
            return false;
        }

        self.has_same_feature(&board_filtered)
    }

    pub fn check_diagonal_lt_rb(&self) -> bool {
        let board_filtered = self
            .board
            .iter()
            .filter(|each| {
                (each.0.row == 0 && each.0.col == 0
                    || each.0.row == 1 && each.0.col == 1
                    || each.0.row == 2 && each.0.col == 2
                    || each.0.row == 3 && each.0.col == 3)
                    && each.1.is_some()
            })
            .collect::<Vec<_>>();

        if board_filtered.len() < 4 {
            return false;
        }

        self.has_same_feature(&board_filtered)
    }

    pub fn check_diagonal_rt_lb(&self) -> bool {
        let board_filtered = self
            .board
            .iter()
            .filter(|each| {
                (each.0.row == 0 && each.0.col == 3
                    || each.0.row == 1 && each.0.col == 2
                    || each.0.row == 2 && each.0.col == 1
                    || each.0.row == 3 && each.0.col == 0)
                    && each.1.is_some()
            })
            .collect::<Vec<_>>();

        if board_filtered.len() < 4 {
            return false;
        }

        self.has_same_feature(&board_filtered)
    }

    pub fn check_game_state(&mut self) -> bool {
        if self.check_diagonal_lt_rb() || self.check_diagonal_rt_lb() {
            return true;
        };

        for n in 0..4 {
            if self.check_row(n) {
                return true;
            };
        }

        for n in 0..4 {
            if self.check_col(n) {
                return true;
            };
        }

        false
    }

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        arrow_instances_data: &HashMap<&'static str, instance::InstanceRaw>,
        arrow_instance_buffer: &wgpu::Buffer,
        circle_instances_data: &HashMap<(i8, i8), instance::InstanceRaw>,
        circle_instance_buffer: &wgpu::Buffer,
        game_pieces: &HashMap<&'static str, (Model, wgpu::Buffer, [f32; 3])>,
        board_coords: &Vec<(Coordinate, [f32; 3])>,
    ) {
        match self.selected_coor {
            Some(coor) => {
                if self.piece_played {
                    match game_pieces.get(&self.selected_piece.name) {
                        Some((_, instance_buffer, board_point)) => {
                            let game_piece_pos;
                            let game_piece_rot;

                            let (_, circle_point) = board_coords
                                .iter()
                                .find(|each| each.0.row == coor.row && each.0.col == coor.col)
                                .unwrap();

                            let new_point = [
                                board_point[0] + circle_point[0],
                                board_point[1] + circle_point[1],
                                board_point[2] + circle_point[2],
                            ];

                            game_piece_pos = cgmath::Vector3::from(new_point);

                            game_piece_rot = if game_piece_pos.is_zero() {
                                cgmath::Quaternion::from_axis_angle(
                                    cgmath::Vector3::unit_z(),
                                    cgmath::Deg(0.0),
                                )
                            } else {
                                cgmath::Quaternion::from_axis_angle(
                                    game_piece_pos.normalize(),
                                    cgmath::Deg(0.0),
                                )
                            };

                            let game_piece_instance_data = instance::Instance {
                                position: game_piece_pos,
                                rotation: game_piece_rot,
                            }
                            .to_raw();

                            queue.write_buffer(
                                &instance_buffer,
                                0,
                                bytemuck::cast_slice(&[game_piece_instance_data]),
                            );

                            self.selected_coor = None;
                        }
                        None => {}
                    }

                    let index = self
                        .available_pieces
                        .iter()
                        .position(|each| each.name == self.selected_piece.name)
                        .unwrap();

                    self.available_pieces.remove(index);

                    if self.available_pieces.len() > 0 {
                        self.selected_piece = self.available_pieces[0];
                    }

                    match self.turn {
                        Turn::Opponent => {
                            match self.board.iter().find(|each| each.1.is_none()) {
                                Some(available_coor) => {
                                    self.ai.select_piece(
                                        &mut self.available_pieces,
                                        &mut self.selected_piece,
                                    );

                                    self.selected_coor = Some(Coordinate {
                                        row: available_coor.0.row,
                                        col: available_coor.0.col,
                                    });

                                    queue.write_buffer(
                                        &circle_instance_buffer,
                                        0,
                                        bytemuck::cast_slice(&[*circle_instances_data
                                            .get(&(available_coor.0.row, available_coor.0.col))
                                            .unwrap()]),
                                    );
                                }
                                None => {}
                            }
                            if self.check_game_state() {
                                println!("{:?} won !!!", self.turn);
                                self.ended = true;
                            }
                            self.turn = Turn::Player;
                        }
                        Turn::Player => {}
                    }

                    self.piece_played = false;
                } else {
                    queue.write_buffer(
                        &circle_instance_buffer,
                        0,
                        bytemuck::cast_slice(&[*circle_instances_data
                            .get(&(coor.row, coor.col))
                            .unwrap()]),
                    );
                }
            }
            None => {}
        }
        queue.write_buffer(
            &arrow_instance_buffer,
            0,
            bytemuck::cast_slice(&[*arrow_instances_data.get(&self.selected_piece.name).unwrap()]),
        );
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        if state == ElementState::Pressed {
            match key {
                VirtualKeyCode::Up => {
                    match self.selected_coor {
                        Some(coor) => {
                            let mut availab_coors_iter =
                                self.board.iter().filter(|each| each.1.is_none());

                            let mut availab_coors_iter_clone = availab_coors_iter.to_owned();

                            match availab_coors_iter
                                .find(|each| each.0.row == coor.row - 1 && each.0.col == coor.col)
                            {
                                Some(availab_coor) => {
                                    // println!("");
                                    self.selected_coor = Some(availab_coor.0);
                                }
                                None => {
                                    // println!("Not Available");

                                    let mut coor_index = availab_coors_iter_clone
                                        .to_owned()
                                        .position(|s| s.0.row == coor.row && s.0.col == coor.col)
                                        .unwrap();

                                    let num_coors = availab_coors_iter_clone.to_owned().count();

                                    coor_index = if coor_index == 0 {
                                        num_coors - 1
                                    } else {
                                        coor_index - 1
                                    };

                                    self.selected_coor =
                                        Some(availab_coors_iter_clone.nth(coor_index).unwrap().0);
                                }
                            }
                        }
                        None => {}
                    }

                    true
                }
                VirtualKeyCode::Right => {
                    match self.selected_coor {
                        Some(coor) => {
                            let mut availab_coors_iter =
                                self.board.iter().filter(|each| each.1.is_none());

                            let mut availab_coors_iter_clone = availab_coors_iter.to_owned();

                            match availab_coors_iter
                                .find(|each| each.0.row == coor.row && each.0.col == coor.col + 1)
                            {
                                Some(availab_coor) => {
                                    // println!("");
                                    self.selected_coor = Some(availab_coor.0);
                                }
                                None => {
                                    // println!("Not Available");

                                    let mut coor_index = availab_coors_iter_clone
                                        .to_owned()
                                        .position(|s| s.0.row == coor.row && s.0.col == coor.col)
                                        .unwrap();

                                    let num_coors = availab_coors_iter_clone.to_owned().count();

                                    coor_index = if coor_index + 1 == num_coors {
                                        0
                                    } else {
                                        coor_index + 1
                                    };

                                    self.selected_coor =
                                        Some(availab_coors_iter_clone.nth(coor_index).unwrap().0);
                                }
                            }
                        }
                        None => {
                            // println!("");
                            // println!("{:?}", self.available_pieces);
                            // println!("{:?}", self.selected_piece);
                            // println!("");
                            if self.available_pieces.len() > 0 {
                                let mut index = self
                                    .available_pieces
                                    .iter()
                                    .position(|&s| s.name == self.selected_piece.name)
                                    .unwrap();

                                let num_pieces = self.available_pieces.len();
                                index = if index + 1 == num_pieces {
                                    0
                                } else {
                                    index + 1
                                };

                                self.selected_piece = self.available_pieces[index];
                                println!("Selected Piece: {:?}", self.selected_piece);
                            }
                        }
                    }

                    true
                }
                VirtualKeyCode::Down => {
                    match self.selected_coor {
                        Some(coor) => {
                            let mut availab_coors_iter =
                                self.board.iter().filter(|each| each.1.is_none());

                            let mut availab_coors_iter_clone = availab_coors_iter.to_owned();

                            match availab_coors_iter
                                .find(|each| each.0.row == coor.row + 1 && each.0.col == coor.col)
                            {
                                Some(availab_coor) => {
                                    // println!("");
                                    self.selected_coor = Some(availab_coor.0);
                                }
                                None => {
                                    // println!("Not Available");

                                    let mut coor_index = availab_coors_iter_clone
                                        .to_owned()
                                        .position(|s| s.0.row == coor.row && s.0.col == coor.col)
                                        .unwrap();

                                    let num_coors = availab_coors_iter_clone.to_owned().count();

                                    coor_index = if coor_index + 1 == num_coors {
                                        0
                                    } else {
                                        coor_index + 1
                                    };

                                    self.selected_coor =
                                        Some(availab_coors_iter_clone.nth(coor_index).unwrap().0);
                                }
                            }
                        }
                        None => {}
                    }

                    true
                }
                VirtualKeyCode::Left => {
                    match self.selected_coor {
                        Some(coor) => {
                            let mut availab_coors_iter =
                                self.board.iter().filter(|each| each.1.is_none());

                            let mut availab_coors_iter_clone = availab_coors_iter.to_owned();

                            match availab_coors_iter
                                .find(|each| each.0.row == coor.row && each.0.col == coor.col - 1)
                            {
                                Some(availab_coor) => {
                                    // println!("");
                                    self.selected_coor = Some(availab_coor.0);
                                }
                                None => {
                                    // println!("Not Available");

                                    let mut coor_index = availab_coors_iter_clone
                                        .to_owned()
                                        .position(|s| s.0.row == coor.row && s.0.col == coor.col)
                                        .unwrap();

                                    let num_coors = availab_coors_iter_clone.to_owned().count();

                                    coor_index = if coor_index == 0 {
                                        num_coors - 1
                                    } else {
                                        coor_index - 1
                                    };

                                    self.selected_coor =
                                        Some(availab_coors_iter_clone.nth(coor_index).unwrap().0);
                                }
                            }
                        }
                        None => {
                            if self.available_pieces.len() > 0 {
                                let mut index = self
                                    .available_pieces
                                    .iter()
                                    .position(|&s| s.name == self.selected_piece.name)
                                    .unwrap();

                                let num_pieces = self.available_pieces.len();
                                index = if index == 0 {
                                    num_pieces - 1
                                } else {
                                    index - 1
                                };

                                self.selected_piece = self.available_pieces[index];
                                println!("Selected Piece: {:?}", self.selected_piece);
                            }
                        }
                    }

                    true
                }
                VirtualKeyCode::Return => {
                    match self.selected_coor {
                        Some(coor) => {
                            self.board
                                .iter_mut()
                                .find(|s| s.0.row == coor.row && s.0.col == coor.col)
                                .unwrap()
                                .1 = Some(self.selected_piece);

                            self.piece_played = true;

                            if self.check_game_state() {
                                println!("{:?} won >_<", self.turn);
                                self.ended = true;
                            }
                        }
                        None => match self.turn {
                            Turn::Player => {
                                self.ai.select_place(
                                    &mut self.board,
                                    self.selected_piece,
                                    &mut self.selected_coor,
                                );
                                self.piece_played = true;
                                self.turn = Turn::Opponent;
                            }
                            Turn::Opponent => {}
                        },
                    }

                    true
                }

                _ => false,
            }
        } else {
            false
        }
    }
}

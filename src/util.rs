

// pub fn __get_RC_values(board, direction) -> i32 {


//     let nb_rows = len(board);
//     let nb_cols = len(board[0]);

//     for i in xrange(nb_rows){
//         val_color = 0
//         val_height = 0
//         val_shape = 0
//         val_state = 0

//         for j in xrange(nb_cols):
//             piece = None
//             if direction == "row":
//                 piece = board[i][j]
//             elif direction == "col":
//                 piece = board[j][i]

//             if piece is not None:
//                 val_color += piece.color_int()
//                 val_height += piece.height_int()
//                 val_shape += piece.shape_int()
//                 val_state += piece.state_int()

//         return  {
//             "color": val_color,
//             "height": val_height,
//             "shape": val_shape,
//             "state": val_state
//         }
//     }
// }


pub fn get_board_values(board) -> i32 {
    let mut values = [];

    values.push(__get_RC_values(board, "row"));
    values.push(__get_RC_values(board, "col"));
    values.push(__get_diag_values(board, "down"));
    values.push(__get_diag_values(board, "up"));

    return values;
}


pub fn maximize_property(board, prop, values) -> i32 {

    if values == None {
        board_values = get_board_values(board);
    }
    else{
        board_values = values;
    }

    let prop_name = prop["propriety"];

    let mut better_index = -1;
    let mut better_pos = None;

    for i in xrange(len(board_values)) {
        let mut better_v = None;

        if better_index > -1 {
            better_v = board_values[better_index];
        }
        let v = board_values[i];

        let mut pos = None;
        if i < 4{
            pos = __get_available_place(board, prop, i, "row");
        }
        else if i < 8 {
            pos = __get_available_place(board, prop, i - 4, "col");
        }
        else if i < 9 {
            pos = __get_available_place(board, prop, None, "diag-down");
        }
        else if i < 10 {
            pos = __get_available_place(board, prop, None, "diag-up");
        }

        if pos == None{
        }
        else if better_v == None || 
                (v[prop_name] ** 2 > better_v[prop_name] ** 2) {
            better_index = i;
            better_pos = pos;
        }
    }
    val_prop = board_values[better_index][prop_name];

    return {
        "position": better_pos,
        "value": val_prop
    }
}

pub fn get_wining_properties(board) -> i32 {
    
    let mut props = set();

    for board_values in get_board_values(board) {
        if board_values["color"] == -3 {
            props.add("Light");
        }
        else if board_values["color"] == 3{
            props.add("Dark");
        }

        if board_values["height"] == -3 {
            props.add("Short");
        }
        else if board_values["height"] == 3 {
            props.add("Tall");
        }

        if board_values["shape"] == -3 {
            props.add("Round");
        }
        else if board_values["shape"] == 3 {
            props.add("Square");
        }

        if board_values["state"] == -3 {
            props.add("Solid");
        }
        else if board_values["state"] == 3 {
            props.add("Hollow");
        }
    }

    return props;
}

pub fn eval_position(board, pos) -> i32 {

    let mut eval_pos = 0;

    piece = board[pos["x"]][pos["y"]];
    board[pos["x"]][pos["y"]] = None;

    board_values = get_board_values(board);

    better_color = maximize_property(
        board,
        {"propriety": "color", "value": piece.color},
        board_values
    );

    better_height = maximize_property(
        board,
        {"propriety": "height", "value": piece.height},
        board_values
    );

    better_shape = maximize_property(
        board,
        {"propriety": "shape", "value": piece.shape},
        board_values
    );

    better_state = maximize_property(
        board,
        {"propriety": "state", "value": piece.state},
        board_values
    );


    if better_color["position"] == pos {
        eval_pos += 1;
    }

    if better_height["position"] == pos {
        eval_pos += 1;
    }

    if better_shape["position"] == pos {
        eval_pos += 1;
    }

    if better_state["position"] == pos {
        eval_pos += 1;
    }

    board[pos["x"]][pos["y"]] = piece;

    winning_props = get_wining_properties(board);
    eval_pos += len(winning_props);

    if ("red" in winning_props && "blue" in winning_props) ||
            ("short" in winning_props && "tall" in winning_props) ||
            ("round" in winning_props && "square" in winning_props) ||
            ("solid" in winning_props && "hollow" in winning_props) {
        //  two different winning values for a same property,
        //  it's like this board will be won on the next turn
        eval_pos = 0;
    }

    return eval_pos;
}
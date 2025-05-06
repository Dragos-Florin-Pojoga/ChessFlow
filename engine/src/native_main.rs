mod chess_lib;
use chess_lib::*;

use std::io::{self, BufRead};


fn main() {
    let mut board = Board::new_start_pos();
    let mut legal_moves = board.generate_legal_moves();

    println!("{}", board);

    let stdin = io::stdin();
    for maybe_line in stdin.lock().lines() {
        match maybe_line {
            Ok(line) => {
                let mv = parse_move_string(&line).unwrap();
                if legal_moves.contains(&mv) {
                    board = board.make_move(&mv);
                    legal_moves = board.generate_legal_moves();
                    println!("{}\nlegal moves available:{}\n\n", board, legal_moves.len());
                }
            }
            Err(err) => {
                eprintln!("error reading stdin: {}", err);
                break;
            }
        }
    }
}

// fn main() {
//     let stdin = io::stdin();
//     for maybe_line in stdin.lock().lines() {
//         match maybe_line {
//             Ok(line) => {
//                 let cmd = parse_command(&line);
//                 match cmd {
//                     Ok(cmd) => {
//                         if let UciCommand::Quit = cmd {
//                             break;
//                         }
//                         println!("{}", execute_command(cmd));
//                     },
//                     Err(_) => {}
//                 }
//             }
//             Err(err) => {
//                 eprintln!("error reading stdin: {}", err);
//                 break;
//             }
//         }
//     }
// }



// fn main() {
//     let start_board = Board::new_start_pos();
//     println!("Initial Board State:");
//     for pt_idx in 0..6 {
//         let pt = PieceType::ALL[pt_idx];
//         println!("{:?}:", pt);
//         println!("{}", start_board.piece_bbs[pt_idx]);
//     }
//     println!("White's pieces:\n{}", start_board.color_bbs[Color::White as usize]);
//     println!("Black's pieces:\n{}", start_board.color_bbs[Color::Black as usize]);
//     println!("All occupied:\n{}", start_board.occupied_bb);
//     println!("Turn: {:?}", start_board.turn);


//     println!("\n--- Pseudo-legal moves for White at start: ---");
//     let pseudo_moves = start_board.generate_pseudo_legal_moves();
//     for (i, mv) in pseudo_moves.iter().enumerate() {
//         println!("{:3}: {:?} to {:?} (Promo: {:?})", i + 1, mv.from, mv.to, mv.promotion);
//     }
//     println!("Total pseudo-legal moves: {}", pseudo_moves.len()); // Should be 20

//     println!("\n--- Legal moves for White at start: ---");
//     let legal_moves = start_board.generate_legal_moves();
//      for (i, mv) in legal_moves.iter().enumerate() {
//         println!("{:3}: {:?} to {:?} (Promo: {:?})", i + 1, mv.from, mv.to, mv.promotion);
//     }
//     println!("Total legal moves: {}", legal_moves.len()); // Should be 20



//     let ep_fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2".parse::<Fen>().unwrap();
//     let ep_board = Board::from_fen(&ep_fen).unwrap();

//     println!("\n--- En Passant Test Board (White to move, EP on d6) ---");
//     println!("White Pawn on E5, Black Pawn on D5, EP target D6");
//     println!("EP target square: {:?}", ep_board.en_passant_square);
//     println!("White's pawn moves from E5:");
//     let ep_pseudo_moves = ep_board.generate_pseudo_legal_moves();
//     for mv in ep_pseudo_moves.iter().filter(|m| m.from == Square::E5) {
//          println!("  Move: {:?} to {:?} (Promo: {:?})", mv.from, mv.to, mv.promotion);
//          if mv.to == Square::D6 {
//              println!("    ^ This is an en passant capture!");
//          }
//     }
    
//     let ep_legal_moves = ep_board.generate_legal_moves();
//     println!("Legal moves for EP test board (filtered for pawn on E5):");
//      for mv in ep_legal_moves.iter().filter(|m| m.from == Square::E5) {
//          println!("  Move: {:?} to {:?} (Promo: {:?})", mv.from, mv.to, mv.promotion);
//     }

//     let castle_fen = "R3K2R/8/8/8/8/8/8/R3K2R w KQkq - 0 1".parse::<Fen>().unwrap();
//     let castle_board = Board::from_fen(&castle_fen).unwrap();

//     println!("\n--- Castling Test Board (White to move) ---");
//     println!("Castling rights: {:?}", castle_board.castling_rights);
//     let castle_legal_moves = castle_board.generate_legal_moves();
//     for mv in castle_legal_moves.iter() {
//         if castle_board.piece_bbs[PieceType::King as usize].is_set(mv.from) && (mv.to.file() as i8 - mv.from.file() as i8).abs() == 2 {
//             println!("  Castling Move: {:?} to {:?}", mv.from, mv.to);
//         }
//     }
// }

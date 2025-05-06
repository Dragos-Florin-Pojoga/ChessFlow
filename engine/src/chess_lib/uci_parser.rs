// https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf#file-uci-protocol-specification-txt
#[derive(Debug)]
pub enum UciCommand {
    Uci,
    SetOption { name: String, value: String },
    IsReady,
    NewGame,
    Position { fen: Option<Fen>, moves: Vec<String> }, // if fen is None, then command is 'position startpos ...'
    Go,
    Stop,
    Quit,
}

// https://www.chess.com/terms/fen-chess
#[derive(Debug, Clone)]
pub struct Fen {
    pub piece_placement: String,
    pub active_color: char,            // 'w' or 'b'
    pub castling_availability: String, // e.g., "KQkq" or "-"
    pub en_passant_target: String,     // e.g., "e3" or "-"
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Fen {
    pub fn to_string(&self) -> String { // proper FEN string
        format!(
            "{} {} {} {} {} {}",
            self.piece_placement,
            self.active_color,
            self.castling_availability,
            self.en_passant_target,
            self.halfmove_clock,
            self.fullmove_number
        )
    }
}

// TODO: the response times could be improved with a proper parser
pub fn parse_command(line: &str) -> Result<UciCommand, String> {
    let mut parts = line.trim().split_ascii_whitespace();
    let command = parts.next().map(|s| s.to_ascii_lowercase());

    // NOTE: in this implementation, the order of the cases WILL impact performance
    Ok(
        match command.as_deref() {
            Some("go")          => UciCommand::Go,

            Some("position")    => {
                // position [fen <FEN-string> | startpos ]  moves <move1> ...
                let mut fen = None;
                let mut moves = Vec::new();

                if let Some(token) = parts.next() {
                    if token == "startpos" {
                        // leave fen = None to indicate startpos
                    } else if token == "fen" {
                        // collect FEN (next 6 fields)
                        let fen_parts: Vec<&str> = parts.by_ref().take(6).collect();
                        
                        // FIXME: this can panick!!!
                        fen = Some(Fen {
                            piece_placement: fen_parts[0].to_string(),
                            active_color: fen_parts[1].chars().next().ok_or("FEN missing active color")?,
                            castling_availability: fen_parts[2].to_string(),
                            en_passant_target: fen_parts[3].to_string(),
                            halfmove_clock: fen_parts[4].parse().map_err(|_| "FEN invalid halfmove clock")?,
                            fullmove_number: fen_parts[5].parse().map_err(|_| "FEN invalid fullmove number")?,
                        });
                    }
                }
                // parse moves if “moves” keyword appears
                if let Some(tok) = parts.next() {
                    if tok == "moves" {
                        moves.extend(parts.map(String::from));
                    }
                }
                UciCommand::Position { fen, moves }
            }

            Some("uci")         => UciCommand::Uci,
            Some("isready")     => UciCommand::IsReady,
            Some("ucinewgame")  => UciCommand::NewGame,
            Some("stop")        => UciCommand::Stop,
            Some("quit")        => UciCommand::Quit,

            Some("setoption")   => {
                // setoption name <name> value <val>
                let mut name = String::new();
                let mut value = String::new();
                while let Some(tok) = parts.next() {
                    match tok {
                        "name"  => if let Some(n) = parts.next() { name = n.to_string(); },
                        "value" => if let Some(v) = parts.next() { value = v.to_string(); },
                        _ => {}
                    }
                }
                UciCommand::SetOption { name, value }
            }

            Some(other)         => Err(format!("Unknown command: '{}'", other))?,
            None                => Err("No command provided".to_string())?,
        }
    )
}
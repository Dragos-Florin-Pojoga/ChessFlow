// TODO: a lot of validation in this file can be removed if we expect a proper environment

// https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf#file-uci-protocol-specification-txt
#[derive(Debug, PartialEq, Eq)]
pub enum UciCommand {
    Uci,
    SetOption { name: String, value: String },
    IsReady,
    NewGame,
    Position { fen: Option<Fen>, moves: Vec<String> }, // if fen is None, then command is 'position startpos ...'
    Go { depth: Option<u8> }, // more can be added
    Stop,
    Quit,
}

// https://www.chess.com/terms/fen-chess
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fen {
    pub piece_placement: String,
    pub active_color: char,            // 'w' or 'b'
    pub castling_availability: String, // e.g., "KQkq" or "-"
    pub en_passant_target: String,     // e.g., "e3" or "-"
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenParseError {
    InvalidInputFormat,
    InvalidPiecePlacement(String),
    InvalidRankCount,
    InvalidFileCountInRank(usize),
    UnknownPieceChar(char),
    InvalidActiveColor(char),
    InvalidCastlingString(String),
    InvalidEnPassantTarget(String),
    InvalidHalfmoveClock(std::num::ParseIntError),
    InvalidFullmoveNumber(std::num::ParseIntError),
    MissingFenFields,
}

use std::fmt;

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenParseError::InvalidInputFormat => write!(f, "Invalid input format for FEN"),
            FenParseError::InvalidPiecePlacement(s) => write!(f, "Invalid piece placement string: {}", s),
            FenParseError::InvalidRankCount => write!(f, "Invalid number of ranks in piece placement (expected 8)"),
            FenParseError::InvalidFileCountInRank(rank) => write!(f, "Invalid number of files in rank {} (expected 8)", rank + 1),
            FenParseError::UnknownPieceChar(c) => write!(f, "Unknown piece character in FEN: {}", c),
            FenParseError::InvalidActiveColor(c) => write!(f, "Invalid active color in FEN: {} (expected 'w' or 'b')", c),
            FenParseError::InvalidCastlingString(s) => write!(f, "Invalid castling availability string: {}", s),
            FenParseError::InvalidEnPassantTarget(s) => write!(f, "Invalid en passant target square string: {}", s),
            FenParseError::InvalidHalfmoveClock(e) => write!(f, "Invalid halfmove clock value: {}", e),
            FenParseError::InvalidFullmoveNumber(e) => write!(f, "Invalid fullmove number value: {}", e),
            FenParseError::MissingFenFields => write!(f, "Missing fields in FEN string"),
        }
    }
}
impl std::error::Error for FenParseError {
    // The default implementation of `source` is sufficient for our ParseIntError variants
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

use std::str::FromStr;

impl FromStr for Fen {
    type Err = FenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_ascii_whitespace().collect();
        if parts.len() != 6 {
            return Err(FenParseError::InvalidInputFormat);
        }

        let piece_placement = parts[0].to_string();
        // Basic validation for piece placement structure (can be expanded)
        if piece_placement.split('/').count() != 8 {
             return Err(FenParseError::InvalidRankCount);
        }
        // More detailed piece placement validation would go here

        let active_color = parts[1].chars().next().ok_or(FenParseError::InvalidActiveColor(' '))?;
        if active_color != 'w' && active_color != 'b' {
            return Err(FenParseError::InvalidActiveColor(active_color));
        }

        let castling_availability = parts[2].to_string();
        // Basic validation for castling string (can be expanded)
        if !castling_availability.chars().all(|c| "KQkq-".contains(c)) {
             return Err(FenParseError::InvalidCastlingString(castling_availability.clone()));
        }


        let en_passant_target = parts[3].to_string();
        // Basic validation for en passant target (can be expanded)
        if en_passant_target != "-" && (en_passant_target.len() != 2 || !en_passant_target.chars().next().unwrap().is_ascii_alphabetic() || !en_passant_target.chars().nth(1).unwrap().is_ascii_digit()) {
             return Err(FenParseError::InvalidEnPassantTarget(en_passant_target.clone()));
        }


        let halfmove_clock = parts[4].parse::<u8>().map_err(FenParseError::InvalidHalfmoveClock)?;
        let fullmove_number = parts[5].parse::<u16>().map_err(FenParseError::InvalidFullmoveNumber)?;

        Ok(Fen {
            piece_placement,
            active_color,
            castling_availability,
            en_passant_target,
            halfmove_clock,
            fullmove_number,
        })
    }
}


/// Error types for UCI command parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UciParseError {
    UnknownCommand(String),
    MissingCommand,
    InvalidPositionCommand,
    MissingFenString,
    FenParseError(FenParseError), // Embed FenParseError
    InvalidSetOptionFormat,
    MissingSetOptionName,
    MissingSetOptionValue,
    MissingFenFields,
}

use std::error::Error; // Import Error trait

impl fmt::Display for UciParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UciParseError::UnknownCommand(cmd) => write!(f, "Unknown command: '{}'", cmd),
            UciParseError::MissingCommand => write!(f, "No command provided"),
            UciParseError::InvalidPositionCommand => write!(f, "Invalid format for 'position' command"),
            UciParseError::MissingFenString => write!(f, "Missing FEN string after 'fen' keyword"),
            UciParseError::MissingFenFields => write!(f, "Missing fields in FEN string"),
            UciParseError::FenParseError(e) => write!(f, "FEN parsing error: {}", e),
            UciParseError::InvalidSetOptionFormat => write!(f, "Invalid format for 'setoption' command"),
            UciParseError::MissingSetOptionName => write!(f, "Missing 'name' keyword for 'setoption' command"),
            UciParseError::MissingSetOptionValue => write!(f, "Missing 'value' keyword for 'setoption' command"),
        }
    }
}

impl Error for UciParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UciParseError::FenParseError(e) => Some(e),
            _ => None,
        }
    }
}


pub fn parse_command(line: &str) -> Result<UciCommand, UciParseError> {
    let mut parts = line.trim().split_ascii_whitespace();
    let command = parts.next().map(|s| s.to_ascii_lowercase());

    match command.as_deref() {
        Some("go")          => {
            if let Some(token) = parts.next() {
                match token {
                    "depth" => {
                        if let Some(depth) = parts.next() {
                            let depth = depth.parse().unwrap_or(0);
                            if depth != 0 {
                                return Ok(UciCommand::Go{ depth: Some(depth) });
                            }
                        }
                    }
                    _ => {}
                }
            }
            return Ok(UciCommand::Go{ depth: None });
        },

        Some("position")    => {
            let mut fen: Option<Fen> = None;
            let mut moves: Vec<String> = Vec::new();
            let mut parsing_moves = false; // State to track if we are parsing moves

            if let Some(token) = parts.next() {
                match token {
                    "startpos" => {
                        // fen remains None
                    }
                    "fen" => {
                        // Collect the next 6 parts for the FEN string
                        let fen_parts: Vec<&str> = parts.by_ref().take(6).collect();
                        if fen_parts.len() != 6 {
                            return Err(UciParseError::MissingFenFields); // Use a specific error
                        }
                        let fen_string = fen_parts.join(" ");
                        fen = Some(fen_string.parse::<Fen>().map_err(UciParseError::FenParseError)?); // Use FromStr for Fen
                    }
                    _ => return Err(UciParseError::InvalidPositionCommand), // Expected 'startpos' or 'fen'
                }
            } else {
                // No token after 'position', which is invalid according to UCI
                return Err(UciParseError::InvalidPositionCommand);
            }

            // After processing startpos or fen, the next token *might* be "moves"
            if let Some(token) = parts.next() {
                if token == "moves" {
                    parsing_moves = true;
                } else {
                    // If it's not "moves", the command format is incorrect
                    return Err(UciParseError::InvalidPositionCommand);
                }
            }

            // If we are parsing moves, collect the rest of the tokens
            if parsing_moves {
                moves.extend(parts.map(String::from));
            }

            Ok(UciCommand::Position { fen, moves })
        }

        Some("uci")         => Ok(UciCommand::Uci),
        Some("isready")     => Ok(UciCommand::IsReady),
        Some("ucinewgame")  => Ok(UciCommand::NewGame),
        Some("stop")        => Ok(UciCommand::Stop),
        Some("quit")        => Ok(UciCommand::Quit),

        Some("setoption")   => {
            // setoption name <name> [value <val>]
            let mut name: Option<String> = None;
            let mut value: Option<String> = None;
            let mut parsing_name = false;
            let mut parsing_value = false;

            // Skip "setoption" and expect "name"
            if let Some(token) = parts.next() {
                if token == "name" {
                    parsing_name = true;
                } else {
                    return Err(UciParseError::MissingSetOptionName);
                }
            } else {
                 return Err(UciParseError::MissingSetOptionName);
            }

            // Collect the option name until "value" or end of line
            let mut current_name_parts = Vec::new();
            while parsing_name {
                if let Some(token) = parts.next() {
                    if token == "value" {
                        parsing_name = false;
                        parsing_value = true;
                        break;
                    } else {
                        current_name_parts.push(token);
                    }
                } else {
                    parsing_name = false; // End of line, no value provided
                }
            }
            name = Some(current_name_parts.join(" "));

            // Collect the option value until end of line
            let mut current_value_parts = Vec::new();
            while parsing_value {
                 if let Some(token) = parts.next() {
                     current_value_parts.push(token);
                 } else {
                     parsing_value = false; // End of line
                 }
            }
            if !current_value_parts.is_empty() {
                value = Some(current_value_parts.join(" "));
            } else {
                 // According to UCI, value is optional for setoption
                 // If "value" was present but no value followed, it might be an error or an empty value
                 // We'll treat an empty value string as the absence of the value field in our current struct
                 // If "value" was encountered, we should capture the value, even if it's empty.
                 // Let's adjust to handle the case where "value" is the last token.
                 if parsing_value {
                     value = Some("".to_string()); // Value keyword was present but nothing followed
                 }
            }


            match (name, value) {
                 (Some(name_str), val_str) => Ok(UciCommand::SetOption { name: name_str, value: val_str.unwrap_or_default() }), // Default to empty string if value is None
                 _ => Err(UciParseError::InvalidSetOptionFormat), // Should not happen with the logic above, but as a safeguard
            }
        }

        Some(other)         => Err(UciParseError::UnknownCommand(other.to_string())),
        None                => Err(UciParseError::MissingCommand),
    }
}
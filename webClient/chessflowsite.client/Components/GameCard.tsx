import React from "react";
import { Chessboard } from "react-chessboard";
import { Chess } from "chess.js";

import '../src/TailwindScoped.css';

interface GameData {
    whiteName: string;
    blackName: string;
    whiteElo: number;
    blackElo: number;
    whiteDeltaElo: number;
    blackDeltaElo: number;
    isWhiteGuest: boolean;
    isBlackGuest: boolean;
    isBotGame: boolean;
    isWhiteReportable: boolean;
    isBlackReportable: boolean;
    format: "Blitz" | "Bullet" | "Classical";
    result: string;
    fen: string;
    pgn: string;
    startTime: string;
}

interface GameCardProps {
    game: GameData;
    onClick?: () => void;
}

function GameCard({ game, onClick }: GameCardProps) {

    const chess = new Chess();
    chess.loadPgn(game.pgn);
    const lastFen = chess.fen();
    const lastMove = chess.history({ verbose: true }).slice(-1)[0];

    const deltaEloContent = (deltaElo: number | null) => {
        return deltaElo === null ? (
            <></>
        ) : (
        <span className={deltaElo > 0 ? "plus-elo" : "minus-elo"}>
            ({deltaElo > 0 ? `+${deltaElo}` : deltaElo})
        </span>
    );
    }

    return (
        <div className="game-card-container" onClick={onClick }>
            <div className="">
                <div className="">{game.blackName}</div>
                <div className="">{game.blackElo} {deltaEloContent(game.blackDeltaElo)}</div>
            </div>
            <div className="chessboard2">
                <Chessboard
                    position={lastFen}
                    arePiecesDraggable={false}
                    boardWidth={200}
                />
            </div>
            <div className="">
                <div className="">{game.whiteName}</div>
                <div className="">{game.whiteElo} {deltaEloContent(game.whiteDeltaElo)}</div>
            </div>

            <div className="">
                Format: <span className="">{game.format}</span>
            </div>

            <div className="">
                Result: <span className="">{(game.result)}</span>
            </div>
            <div className="">
                Start Time: <span className="">{new Date(game.startTime).toString()}</span>
            </div>
        </div>
    );
}

export default GameCard;
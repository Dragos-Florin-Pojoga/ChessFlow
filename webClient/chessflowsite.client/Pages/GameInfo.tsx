import React, { useState, useEffect, useMemo } from 'react';
import { useParams, useNavigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/Navbar.tsx";
import { getToken } from "../Utils/authToken.js";
import UserStore from '../Stores/UserStore.js';
import { Chessboard } from 'react-chessboard';
import { Chess } from "chess.js";
import PlayerInfoCard from '../Components/PlayerInfoCard.js'; 
import Engine from '../Utils/Engine.js';

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

    format: "Blitz" | "Bullet" | "Classical"; // restrict to known types
    result: string;
    fen: string;
    pgn: string;
    startTime: string;
}

function GameInfo() {
    const param = useParams();
    const [exists, setExists] = useState(false);
    const [game, setGame] = useState<GameData | null>(null);

    const engine = useMemo(() => new Engine(), []);
    const [engineGame, setEngineGame] = useState(new Chess());
    const [moveHistory, setMoveHistory] = useState<string[]>([]);
    const [currentMove, setCurrentMove] = useState(0);

    const [chessBoardPosition, setChessBoardPosition] = useState(engineGame.fen());
    const [positionEvaluation, setPositionEvaluation] = useState(0);
    const [depth, setDepth] = useState(10);
    const [bestLine, setBestline] = useState("");
    const [possibleMate, setPossibleMate] = useState("");
    const findBestMove = () => {
        console.log("test");
        engine.evaluatePosition(chessBoardPosition, 18);
        console.log("test2");
        engine.onMessage(({
            positionEvaluation,
            possibleMate: newMate,
            pv,
            depth: newDepth
        }) => {
            console.log("test3");
            if (depth && depth < 10) return;
            if (positionEvaluation) {
                const numericEval = (engineGame.turn() === "w" ? 1 : -1) * Number(positionEvaluation) / 100;
                setPositionEvaluation(numericEval);
            }
            if (newMate !== undefined) {
                setPossibleMate(newMate);
            } else {
                setPossibleMate("");
            }

            if (newDepth) setDepth(newDepth);
            if (pv) setBestline(pv);
        });
    }

    const [boardOrientation, setBoardOrientation] = useState<"white" | "black">("white");

    const navigate = useNavigate();

    const { user, setUser } = UserStore();

    const gameID = param.id;

    const token = getToken();

    const CENTIPAWN_BAR_HEIGHT = 400;
    const [evalScore, setEvalScore] = useState<number | null>(null); // positive = white advantage


    const goToMove = (index: number) => {
        const newGame = new Chess();
        const allMoves = [...moveHistory];
        newGame.reset();
        for (let i = 0; i < index; i++) {
            newGame.move(allMoves[i]);
        }
        setEngineGame(newGame);
        setCurrentMove(index);
        setGame({
            ...game,
            fen: newGame.fen(),
        });
        setChessBoardPosition(newGame.fen());
    };

    const getPlayerCardInfo = (isTop: boolean) => {
        const isWhiteBottom = boardOrientation === "white";
        const amIWhite = isTop ? !isWhiteBottom : isWhiteBottom;

        const name = amIWhite ? game?.whiteName : game?.blackName;
        const elo = amIWhite ? game?.whiteElo : game?.blackElo;
        const deltaElo = amIWhite ? game?.whiteDeltaElo : game?.blackDeltaElo;
        const side = amIWhite ? "w" : "b";
        const isGuest = amIWhite ? game?.isWhiteGuest : game?.isBlackGuest;
        const timer = null;
        const isCountingDown = null;

        const isSelf = !isGuest && user.name === name;
        const centered = true;

        return { name, elo, deltaElo, side, isGuest, isSelf, timer, isCountingDown, centered };
    }

    useEffect(() => {

        fetch(`/api/game/show/${gameID}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            }
        })
            .then(async (response) => {
                if (response.ok) {
                    let data = await response.json();
                    setGame(data);

                    const newGame = new Chess();
                    newGame.loadPgn(data.pgn);
                    setEngineGame(newGame);
                    setMoveHistory(newGame.history());
                    setCurrentMove(newGame.history().length);
                    setChessBoardPosition(newGame.fen());

                    setExists(true);
                }
            });
    }, [gameID]);

    useEffect(() => {
        if (!engineGame.isGameOver() || engineGame.isDraw()) {
            findBestMove();
        }
    }, [chessBoardPosition, engineGame]);
    const bestMove = bestLine?.split(" ")?.[0];

    return (
        <AuthorizeView>
            <NavBar></NavBar>
            <>
                {exists ?
                    <div>
                        <h4>
                            Position Evaluation:{" "}
                            {possibleMate
                                ? `Mate in ${possibleMate}`
                                : `${positionEvaluation > 0 ? "+" : ""}${positionEvaluation.toFixed(2)}`
                            }
                            {"; "}
                            Depth: {depth}
                        </h4>
                        <h5>
                            Best line: <i>{bestLine.slice(0, 40)}</i> ...
                        </h5>
                        <h3>Result: {game?.result}</h3>
                        <h5>Format : {game?.format}</h5>
                        <h5>Start time: {new Date(game.startTime).toString()}</h5>
                        <PlayerInfoCard
                            key={boardOrientation === "white" ? "white" : "black"} 
                            gameID={toString(gameID)}
                            {...getPlayerCardInfo(true)} // Top
                        />
                        <div className="eval-container">
                            <div className="eval-bar">
                                <div
                                    className="eval-mis"
                                    style={{
                                        height: `${Math.max(100 - Math.min(Math.max((0.5 + (positionEvaluation ?? 0) / 10) * 100, 0), 100), 0)}%`,
                                    }}
                                />
                                <div
                                    className="eval-value"
                                    style={{
                                        height: `${Math.min(Math.max((0.5 + (positionEvaluation ?? 0) / 10) * 100, 0), 100)}%`,
                                    }}
                                />
                            </div>
                            <div className="chessboard">
                                <Chessboard position={game.fen} boardWidth="400" arePiecesDraggable={false} boardOrientation={boardOrientation} customArrows={bestMove ? [[(bestMove.substring(0, 2) as Square), (bestMove.substring(2, 4) as Square), "rgb(0, 128, 0)"]] : undefined} />
                            </div>   
                        </div>
                        <PlayerInfoCard
                            key={boardOrientation === "white" ? "black" : "white"} 
                            gameID={toString(gameID)}
                            {...getPlayerCardInfo(false)} // Bottom
                        />
                        <button className="flip-button-container2" onClick={() => { setBoardOrientation(boardOrientation === "white" ? "black" : "white"); }}>Flip orientation</button>
                        <div className="w-full md:w-1/2">
                            <h3 className="text-lg font-semibold mb-2">Move List</h3>
                            <div className="grid grid-cols-2 gap-2 text-sm bg-gray-100 p-2 rounded-md border">
                                {moveHistory.map((move, i) => {
                                    const moveNum = Math.floor(i / 2) + 1;
                                    const display = i % 2 === 0 ? `${moveNum}. ${move}` : move;
                                    const isCurrent = i + 1 === currentMove;

                                    return (
                                        <span
                                            key={i}
                                            className={`select-none ${isCurrent ? "bolded-move" : ""
                                                }`}
                                        >
                                            {display}
                                        </span>
                                    );
                                })}
                            </div>
                            <div className="mt-4 flex gap-2">
                                <button
                                    onClick={() => goToMove(0)}
                                    className="px-3 py-1 bg-gray-300 rounded hover:bg-gray-400"
                                >
                                    Start
                                </button>
                                <button
                                    onClick={() => goToMove(Math.max(currentMove - 1, 0))}
                                    className="px-3 py-1 bg-gray-300 rounded"
                                >
                                    &lt;-
                                </button>
                                <button
                                    onClick={() => goToMove(Math.min(currentMove + 1, moveHistory.length))}
                                    className="px-3 py-1 bg-gray-300 rounded"
                                >
                                    -&gt;
                                </button>
                                <button
                                    onClick={() => goToMove(moveHistory.length)}
                                    className="px-3 py-1 bg-gray-300 rounded hover:bg-gray-400"
                                >
                                    End
                                </button>
                            </div>
                        </div>
                    </div>
                    :
                    <h1>Game doesn't exist!</h1>
                }
            </>

        </AuthorizeView >
    );
}


export default GameInfo;
import React, { useState, useEffect, useRef } from "react";
import { useParams, useNavigate, Navigate } from "react-router-dom";
import UserStore from '../Stores/UserStore.js';
import NavBar from "../Components/NavBar.js";
import SignalRStore from '../Stores/SignalRStore.js';
import GameStore from '../Stores/GameStore.js';
import { isLoggedIn } from '../Utils/authToken.js';
import { connect } from "http2";
import { Chessboard } from '../Vendors/react-chessboard/index.esm.js'; 
import { Chess } from 'chess.js'
import PlayerInfoCard from '../Components/PlayerInfoCard.tsx'; 

import '../src/TailwindScoped.css';

function ChessGame() {
    const param = useParams();
    const gameId = param.id;

    const DEV_TEST: boolean = true; // Set to true for development testing so we can test the game page without a real game

    const [status, setStatus] = useState<string>("");
    const [errors, setErrors] = useState<string[]>([]);

    const { connection, startConnection, stopConnection } = SignalRStore();
    const { user, setUser, clearUser } = UserStore();
    const { game, updateGame } = GameStore();
    const isValidId: boolean = (game != null && 'id' in game && game.id === parseInt(gameId));

    const [selfTimer, setSelfTimer] = useState(game?.timer ?? 0);
    const [opponentTimer, setOpponentTimer] = useState(game?.opponentTimer ?? 0);

    const [moveHistory, setMoveHistory] = useState<Move[]>([]);
    console.log(moveHistory);

    if (!isValidId && !DEV_TEST) {
        return (
            <>
                <Navigate to="/" replace={true} />
            </>
        ); 
    }

    const [boardOrientation, setBoardOrientation] = useState<"white" | "black">(
        game?.side === "b" ? "black" : "white"
    );

    console.log(game);

    useEffect(() => {
        if (game) {
            setSelfTimer(game.timer);
            setOpponentTimer(game.opponentTimer);


        }
        const interval = setInterval(() => {
            if (!game) return;

            if (game.activeSide === game.side) {
                setSelfTimer(prev => Math.max(prev - 1, 0));
            } else {
                setOpponentTimer(prev => Math.max(prev - 1, 0));
            }
        }, 1000);

        return () => clearInterval(interval);
    }, [game]);

    const setError = (e: string) => setErrors([e]);

    const navigate = useNavigate();

    const [clientGame, setClientGame] = useState(new Chess());
    const [moveFrom, setMoveFrom] = useState("");
    const [moveTo, setMoveTo] = useState<Square | null>(null);
    const moveToProxy = useRef(moveTo); // useRef to keep the same reference for the moveTo value)
    const [showPromotionDialog, setShowPromotionDialog] = useState(false);
    const [rightClickedSquares, setRightClickedSquares] = useState({});
    const [moveSquares, setMoveSquares] = useState({});
    const [optionSquares, setOptionSquares] = useState({});

    const invertSide = (side: "w" | "b") => { return side === "w" ? "b" : "w"; }

    const getPlayerCardInfo = (isTop: boolean) => {
        const isWhiteBottom = boardOrientation === "white";
        const isSelfBottom = (game.side === "w") === isWhiteBottom;
        const isSelf = isTop !== isSelfBottom;

        const name = isSelf ? game.name : game.opponentName;
        const elo = isSelf ? game.elo ?? 1500 : game.opponentElo ?? 1500;
        const side = isSelf ? game.side : invertSide(game.side);
        const isGuest = isSelf ? game.isGuest : game.isOpponentGuest;
        const timer = isSelf ? selfTimer : opponentTimer;
        const isCountingDown = game.activeSide === side;
        const key = boardOrientation;

        return {key, name, elo, side, isGuest, isSelf, timer, isCountingDown };
    }
    function safeGameMutate(modify) {
        setClientGame(g => {
            const update = {
                ...g
            };
            modify(update);
            return update;
        });
    }
    function getMoveOptions(square) {
        const moves = clientGame.moves({
            square,
            verbose: true
        });
        if (moves.length === 0) {
            setOptionSquares({});
            return false;
        }
        const newSquares = {};
        moves.map(move => {
            newSquares[move.to] = {
                background: clientGame.get(move.to) && clientGame.get(move.to).color !== clientGame.get(square).color ? "radial-gradient(circle, rgba(0,0,0,.1) 85%, transparent 85%)" : "radial-gradient(circle, rgba(0,0,0,.1) 25%, transparent 25%)",
                borderRadius: "50%"
            };
            return move;
        });
        newSquares[square] = {
            background: "rgba(255, 255, 0, 0.4)"
        };
        setOptionSquares(newSquares);
        return true;
    }

    function clientValidateAndSetMove(square: Square) {
        console.log("Here?");
        // check if valid move before showing dialog
        const moves = clientGame.moves({
            moveFrom,
            verbose: true
        });
        console.log(moves);
        console.log(square);
        const foundMove = moves.find(m => m.from === moveFrom && m.to === square);
        // not a valid move
        if (!foundMove) {
            // check if clicked on new piece
            const hasMoveOptions = getMoveOptions(square);
            // if new piece, setMoveFrom, otherwise clear moveFrom
            setMoveFrom(hasMoveOptions ? square : "");
            return false;
        }

        // valid move
        setMoveTo(square);
        moveToProxy.current = square; // update the ref to the new moveTo value
        console.log(moveToProxy.current);

        // if promotion move
        if (foundMove.color === "w" && foundMove.piece === "p" && square[1] === "8" || foundMove.color === "b" && foundMove.piece === "p" && square[1] === "1") {
            console.log(moveToProxy.current);
            setShowPromotionDialog(true);
            
            return false;
        }

        // is normal move
        const gameCopy = new Chess(clientGame.fen());
        console.log(gameCopy);
        const move = gameCopy.move({
            from: moveFrom,
            to: square,
            promotion: "q"
        });

        // if invalid, setMoveFrom and getMoveOptions
        if (move === null) {
            const hasMoveOptions = getMoveOptions(square);
            if (hasMoveOptions) setMoveFrom(square);
            return false;
        }
        connection?.invoke("MakeMove", game?.id ?? 1, move.san);
        setMoveHistory([...moveHistory, move]);
        setClientGame(gameCopy);
        setMoveFrom("");
        setMoveTo(null);
        setOptionSquares({});
        return true;
    }
    function onSquareClick(square, piece) {
        // only allow clicking if piece belongs to current player or if moveFrom is set
        if (!moveFrom && piece && piece[0] !== (game ? game.side : clientGame.turn())) return;


        // from square
        if (!moveFrom) {
            const hasMoveOptions = getMoveOptions(square);
            if (hasMoveOptions) setMoveFrom(square);
            return;
        }

        // to square
        if (!moveTo) {
            clientValidateAndSetMove(square);
        }
    }
    function onPromotionPieceSelect(piece) {
        // if no piece passed then user has cancelled dialog, don't make move and reset
        console.log(moveToProxy.current);
        if (piece) {
            const gameCopy = new Chess(clientGame.fen());
            const move = gameCopy.move({
                from: moveFrom,
                to: moveToProxy.current,
                promotion: piece[1].toLowerCase() ?? "q"
            });
            connection?.invoke("MakeMove", game?.id ?? 1, move.san);
            setMoveHistory([...moveHistory, move]);
            setClientGame(gameCopy);
        }
        setMoveFrom("");
        setMoveTo(null);
        setShowPromotionDialog(false);
        setOptionSquares({});
        return true;
    }

    function isDraggablePiece({ piece, sourceSquare }) {
        // only allow dragging if piece belongs to current player
        if (piece[0] !== (game ? game.side : clientGame.turn())) return false;

        // Check if there are valid moves for this piece
        const validMoves = clientGame.moves({ square: sourceSquare });
        return validMoves.length > 0;
    }
    function onPieceDragBegin(piece, sourceSquare) {
        const hasMoveOptions = getMoveOptions(sourceSquare);
        if (hasMoveOptions) setMoveFrom(sourceSquare);
        return;
    }

    function onPieceDrop(sourceSquare: Square, targetSquare: Square, piece: Piece) {
        console.log("anyLogic?");
        if (!moveTo) {
            return clientValidateAndSetMove(targetSquare);
        }
        return false;
    }

    useEffect(() => {
        if (!connection) return;
    }, [connection]);

    const setStatusUtil = (message: string) => {
        if (errors.length == 0) {
            setStatus(message);
        }
    };

    return (
        <>
            <NavBar></NavBar>
            <div>
                <>
                    {
                        game && (
                            <PlayerInfoCard
                                gameID={game.id}
                                {...getPlayerCardInfo(true)} // Top
                            />
                        )
                    }
                </>
                <Chessboard boardWidth={400} animationDuration={0} arePiecesDraggable={true} position={clientGame.fen()} onSquareClick={onSquareClick} onPromotionPieceSelect={onPromotionPieceSelect} customBoardStyle={{
                    borderRadius: "4px",
                    boxShadow: "0 2px 10px rgba(0, 0, 0, 0.5)"
                }} customSquareStyles={{
                    ...moveSquares,
                    ...optionSquares,
                    ...rightClickedSquares
                }} promotionToSquare={moveTo} showPromotionDialog={showPromotionDialog} autoPromoteToQueen={false}
                    onPieceDragBegin={onPieceDragBegin} onPieceDrop={onPieceDrop} isDraggablePiece={isDraggablePiece} boardOrientation={boardOrientation === "white" ? "white" : "black"} />
                <>
                    {
                        game && (
                            <PlayerInfoCard
                                gameID={game.id}
                                {...getPlayerCardInfo(false)} // Bottom
                            />
                        )
                    }
                </>
                <button className="flip-button-container" onClick={() => { setBoardOrientation(boardOrientation === "white" ? "black" : "white"); }}>Flip orientation</button>
                <div className="move-list-container p-4 max-w-md mx-auto">
                    <h3 className="font-semibold mb-2">Moves:</h3>
                    <div className="flex flex-wrap gap-x-4 gap-y-2 text-sm bg-gray-50 rounded-md p-2 border">
                        {
                            // Group semimoves in pairs per full move
                            moveHistory.reduce((acc, move, index) => {
                                if (index % 2 === 0) {
                                    // start a new full move pair
                                    acc.push([move]);
                                } else {
                                    // push second semimove into last pair
                                    acc[acc.length - 1].push(move);
                                }
                                return acc;
                            }, []).map((fullMove, i) => {
                                const turn = i + 1;
                                return (
                                    <span key={i} className="flex space-x-1">
                                        <span className="font-medium">{turn}. </span>
                                        <span>{fullMove[0]?.san} </span>
                                        {fullMove[1] && <span>{fullMove[1].san} </span>}
                                    </span>
                                )
                            })
                        }
                    </div>
                </div>
            </div>
        </>

    );
}

export default ChessGame;
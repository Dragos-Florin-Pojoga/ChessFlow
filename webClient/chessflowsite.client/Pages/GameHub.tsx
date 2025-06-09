import React, { useState, useEffect, use } from "react";
import { useNavigate } from "react-router-dom";
import UserStore from '../Stores/UserStore.ts';
import NavBar from "../Components/Navbar.tsx";
import SignalRStore from '../Stores/SignalRStore.ts';
import GameStore from '../Stores/GameStore.ts';
import { isLoggedIn } from '../Utils/authToken.ts';
import { connect } from "http2";

import '../src/TailwindScoped.css';

function GameHub() {
    const [guestElo, setGuestElo] = useState<number>(1200);
    const [guestName, setGuestName] = useState<string>("");
    const [format, setFormat] = useState<string>("Classical");
    const [isBot, setIsBot] = useState<boolean>(false);
    const [botId, setBotId] = useState<number | null>(null);

    const [status, setStatus] = useState<string>("");
    const [errors, setErrors] = useState<string[]>([]);

    const { connection, startConnection, stopConnection } = SignalRStore();
    const { user, setUser, clearUser } = UserStore();
    const { game, setGame } = GameStore();

    const setError = (e: string) => setErrors([e]);

    const navigate = useNavigate();

    useEffect(() => {
        if (!connection) return;

        const handleError = (message: string) => {
            setError(message);
            setStatus("");
        };
        const handleGameStarted = (gameId: number, gameData: any) => {
            console.log(gameData);
            setError(`Game with ID ${gameId} started as ${gameData.side} matched with ${gameData.opponentName}!`);
            setStatus("");
            setGame({
                id: gameId,
                side: gameData.side as 'w' | 'b',
                activeSide: "w", // starts with white, will be updated by server
                name: gameData.name,
                opponentName: gameData.opponentName,
                elo: gameData.elo,
                opponentElo: gameData.opponentElo,
                isGuest: gameData.isGuest,
                isOpponentGuest: gameData.isOpponentGuest,
                isBot: gameData.isBotGame,
                format: gameData.format,
                timer: gameData.timer,
                opponentTimer: gameData.opponentTimer,
                fen: gameData.fen,
                moveHistory: [] 
            });
            console.log(game);
            // Navigate to the game page
            navigate(`/game/play/${gameId}`, { replace: true });
        }

        connection.on("Error", handleError);
        connection.on("GameStarted", handleGameStarted);

        return () => {
            connection.off("Error", handleError); // cleanup on unmount
            connection.off("GameStarted", handleGameStarted);
        };
    }, [connection]);

    const setStatusUtil = (message: string) => {
        if (errors.length == 0) {
            setStatus(message);
        }
    };

    const handleJoinQueue = async () => {
        if (!connection) {
            setError("Not connected to game hub.");
            setStatus("");
            return;
        }

        setErrors([]);

        if (!isLoggedIn()) {
            if (!guestName || guestName.length > 31) {
                setError("Guest name is required and must be under 32 characters.");
                setStatus("");
                return;
            }

            try {
                await connection.invoke("JoinQueue", {
                    guestElo: guestElo,
                    guestUsername: guestName.trim(),
                    format: format,
                    isBot: isBot,
                    botId: isBot ? botId : null
                });
                setStatusUtil(`Attempting to queue as guest ${guestName}.`);
            } catch (err) {
                setError("Failed to join queue: " + err?.toString());
                setStatus("");
            }
            return;
        }
        try {
            await connection.invoke("JoinQueue", {
                guestElo: null,
                guestUsername: null,
                format: format,
                isBot: isBot,
                botId: isBot ? botId : null
            });
            setStatusUtil(`Attempting to queue as ${user.name}.`);
        } catch (err) {
            setError("Failed to join queue: " + err?.toString());
            setStatus("");
        }
    };

    return (
        <>
            <NavBar></NavBar>
            <div className="max-w-md mx-auto p-4 shadow-md rounded bg-white">
                <h2 className="text-2xl font-semibold mb-4">Game Hub</h2>

                <div className="mb-4">
                    {!isLoggedIn() && (
                        <>
                            <h3 className="text-lg font-medium">Guest Info</h3>
                            <div className="mt-2">
                                <label className="block text-sm font-medium">Name</label>
                                <input
                                    type="text"
                                    value={guestName}
                                    onChange={(e) => setGuestName(e.target.value)}
                                    className="w-full p-2 border rounded mt-1"
                                />
                            </div>
                            <div className="mt-2">
                                <label className="block text-sm font-medium">Starting Elo: </label>
                                <select id="elo" name="elo" value={guestElo} onChange={(e) => setGuestElo(parseInt(e.target.value))} defaultValue={1200}>
                                    <option value={400}>New: 400</option>
                                    <option value={800}>Beginner: 800</option>
                                    <option value={1200}>Intermediate: 1200</option>
                                    <option value={1600}>Advanced: 1600</option>
                                    <option value={2000}>Expert: 2000</option>
                                </select>
                            </div>
                        </>
                    )}
                    <div className="mt-2">
                        <label className="block text-sm font-medium">Time format: </label>
                        <select id="format" name="format" value={format} onChange={(e) => setFormat(e.target.value)} defaultValue={"Classical"}>
                            <option value={"Classical"}>Classical</option>
                            <option value={"Blitz"}>Blitz</option>
                            <option value={"Bullet"}>Bullet</option>
                        </select>
                    </div>
                    <div>
                        <input
                            type="checkbox"
                            id="bot"
                            name="bot"
                            checked={isBot}
                            onChange={(e) => { setIsBot(e.target.checked); if (e.target.checked) setBotId(null);}}
                        />
                        <label htmlFor="bot">Play against bot.</label>
                    </div>
                    <div>
                        <label className="forminput" htmlFor="botId">Choose bot:</label>
                    </div>
                    <div>
                        <select id="botId" name="botId" value={botId ?? 0} onChange={(e) => setBotId(parseInt(e.target.value))} defaultValue={0} disabled={!isBot}>
                            <option value={0}>ChessFlow Engine</option>
                        </select>
                    </div>
                </div>

                <button
                    onClick={handleJoinQueue}
                    className="bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 transition"
                >
                    Join Queue
                </button>

                {status && (
                    <div>
                        <p>{status}</p>
                    </div>
                )}

                {errors.length > 0 && (
                    <div className="error">
                        {errors.map((err, index) => (
                            <p key={index}>{err}</p>
                        ))}
                    </div>
                )}
            </div>
        </>
        
    );
}

export default GameHub;
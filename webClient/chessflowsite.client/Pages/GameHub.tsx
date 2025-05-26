import React, { useState, useEffect, use } from "react";
import UserStore from '../Stores/UserStore.ts';
import NavBar from "../Components/NavBar.tsx";
import SignalRStore from '../Stores/signalRStore.ts';
import { isLoggedIn } from '../Utils/authToken.ts';
import { connect } from "http2";

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

    const setError = (e: string) => setErrors([e]);

    useEffect(() => {
        if (!connection) return;

        const handleError = (message: string) => {
            setError(message);
            setStatus("");
        };

        const handleGameStarted = (gameId: number, side: string, opponentName: string) => {
            setError(`Game with ID ${gameId} started as ${side} matched with ${opponentName}!`);
            setStatus("");
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
                            <option value={"Bullet"}>Bullet</option>
                            <option value={"Blitz"}>Blitz</option>
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
import React, { useEffect, useState, startTransition } from "react";
import { Link } from "react-router-dom";
import { useNavigate } from "react-router-dom";
import GameStore from '../Stores/GameStore.ts';

import '../src/TailwindScoped.css';

interface GamePopupProps {
    result: "win" | "loss" | "draw" | "stalemate" | "timeout" | "opponentTimeout" | "resignation" | "opponentResignation";
    whiteElo: number;
    blackElo: number;
    deltaWhiteElo: number | null,
    deltaBlackElo: number | null,
    whiteName: string;
    blackName: string;
}

const resultMessages: Record<string, string> = {
    win: "You Win!",
    loss: "You Lose!",
    draw: "It's a Draw!",
    stalemate: "Stalemate!",
    timeout: "You Ran Out of Time!",
    opponentTimeout: "Opponent Ran Out of Time!",
    resignation: "You Resigned!",
    opponentResignation: "Opponent Resigned!"
};

function GamePopup({ result, whiteElo, blackElo, deltaWhiteElo, deltaBlackElo, whiteName, blackName }: GamePopupProps) {

    const navigate = useNavigate();

    const {clearGame} = GameStore();

    const deltaEloContent = (deltaElo) => { return deltaElo === null ? (
        <></>
    ) : (
        <span className={deltaElo >= 0 ? "plus-elo" : "minus-elo"}>
            ({deltaElo > 0 ? `+${deltaElo}` : deltaElo})
        </span>
    );
};

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center popup2">
            <div className="bg-white rounded-2xl p-6 shadow-xl text-center max-w-sm w-full">
                <h2 className="text-2xl font-bold mb-4">
                    {resultMessages[result]}
                </h2>
                <div>
                    <div className={`w-4 h-4 rounded-full black-circle`} />
                    <div>
                        <div>{blackName}</div>
                        <div>Elo: {blackElo}{deltaEloContent(deltaBlackElo)}</div>
                    </div>
                </div>
                <div>
                    <div className={`w-4 h-4 rounded-full white-circle`} />
                    <div>
                        <div>{whiteName}</div>
                        <div>Elo: {whiteElo}{deltaEloContent(deltaWhiteElo)}</div>
                    </div>
                </div>
                <button
                    onClick={() => {
                        navigate("/", { replace: true });
                        setTimeout(() => clearGame(), 100);
                    }}
                    className="mt-4 px-4 py-2 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition"
                >
                    Close
                </button>
            </div>
        </div>
    );
};

export default GamePopup;
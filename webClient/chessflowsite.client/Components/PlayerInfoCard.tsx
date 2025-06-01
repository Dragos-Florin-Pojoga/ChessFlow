import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { useNavigate } from "react-router-dom";

import '../src/TailwindScoped.css';

interface PlayerInfoCardProps {
    key: any,
    name: string;
    elo: number;
    deltaElo: number | null;
    side: "w" | "b";
    isGuest: boolean;
    isSelf: boolean;
    gameID: number;
    timer: number;
    isCountingDown: boolean;
    centered: boolean;
}

function PlayerInfoCard({key, name, elo, deltaElo = null, side, isGuest, gameID, isSelf, timer, isCountingDown, centered=false} : PlayerInfoCardProps){
    const sideColor = side === "w" ? "white-circle" : "black-circle";

    const [localTimer, setLocalTimer] = useState(timer);

    const navigate = useNavigate();

    const handleReportClick = () => {
        navigate(`/report/${name}?gameID=${gameID}`);
    };

    const formatTime = (seconds: number) => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${mins}:${secs.toString().padStart(2, "0")}`;
    }

    useEffect(() => {
        setLocalTimer(timer); // Sync when timer prop changes
    }, [timer]);

    useEffect(() => {
        if (!isCountingDown) return;

        const interval = setInterval(() => {
            setLocalTimer((prev) => Math.max(prev - 1, 0));
        }, 1000);

        return () => clearInterval(interval);
    }, [isCountingDown]);


    const nameContent = isGuest ? (
        <span className="font-medium">{name}</span>
    ) : (
        <>
            <Link
                to={`/user/${name}`}
                className="text-blue-600 hover:underline font-medium"
            >
                {name}
                </Link>
                <div>
                    {
                        !isSelf && <button type="button" onClick={handleReportClick}>Report!</button>
                    }
                </div>
        </>
    );

    const deltaEloContent = deltaElo === null ? (
        <></>
    ) : (
        <span className={deltaElo > 0 ? "plus-elo" : "minus-elo"}>
            ({deltaElo > 0 ? `+${deltaElo}` : deltaElo})
        </span>
    );

    return (
        <div className={`player-info-card ${isCountingDown ? 'active' : ''} ${centered ? 'centered-player-info-card' : ''}` }>
            <div className={`w-4 h-4 rounded-full ${sideColor}`} />
            <div>
                <div>{nameContent}</div>
                <div>Elo: {elo}{deltaEloContent}</div>
                { timer && (
                    <div className="text-gray-500 text-sm">
                        {formatTime(localTimer)}
                    </div>
                )
                }
            </div>
        </div>
    );
};

export default PlayerInfoCard;
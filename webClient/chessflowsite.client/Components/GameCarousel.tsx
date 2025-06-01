import React, { useCallback, useEffect, useState, useRef, cache } from "react";
import { useNavigate } from "react-router-dom";
import { getToken } from "../Utils/authToken.ts";
import useEmblaCarousel from "embla-carousel-react";
import GameCard from "./GameCard";

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
interface GameDataEntry {
    game: GameData;
    gameId: string;
}

interface GameCarouselProps {
    username: string;
}

const pageSize = 2;
function GameCarousel({ username }: GameCarouselProps){
    const [games, setGames] = useState<GameDataEntry[]>([]);
    const [page, setPage] = useState(1);
    const [lastPage, setlastPage] = useState(0);
    const [ind, setInd] = useState(0);

    const hasFetched = useRef(false); // to guard against strict mode fetching twice \

    const token = getToken();

    const navigate = useNavigate();

    const fetchGames = useCallback(async (pageNumber: number) => {

        const params = new URLSearchParams({
            page: "" + pageNumber,
            pageSize: "" + pageSize,
            usernameOne: username,
            sortType: "date",
            isAscending: "false"
        });

        const response = await fetch(`/api/game/index?${params.toString()}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`
            }
        });

        if (response.ok) {
            const data = await response.json();
            console.log(data);
            setGames(prevGames => [...prevGames, ...data.items]);
            setlastPage(data.lastPage);
            if (pageNumber!=1 && data.items.length > 0) {
                setInd(prevInd => prevInd + 1);
                setPage(pageNumber + 1); 
            }
        } else {
            console.error('Failed to fetch reports');
        }
    }, []);

    useEffect(() => {
        if (!hasFetched.current) {
            hasFetched.current = true;
            fetchGames(1);
        }
    }, [fetchGames]);

    //console.log(games);
    //console.log(ind);
    //console.log(ind + pageSize);

    const visibleGames = games.slice(ind, ind + pageSize);
    //console.log(visibleGames);

    const scrollNext = async () => {
        if (!(page === lastPage) && ind == games.length - 1) return;
        if (!(page >= lastPage) && games.length - ind == pageSize) {
            await fetchGames(page + 1);
        }
        else {
            setInd(ind + 1);
        }
    };

    const scrollPrev = () => {
        if (ind > 0) {
            setInd(ind - 1);
        }
    };

    useEffect(() => {
        console.log(ind);
    }, [ind]);


    if (games.length === 0) {
        return (
            <div className="mt-4 text-center text-gray-500">
                <p>No games found.</p>
            </div>
        );
    }

    return (
        <div>
            <div className="carousel-div">
                <div className="game-carousel-container ">
                    {visibleGames.map(({ game, gameId }) => (
                        <div key={gameId} className="embla__slide min-w-[300px] max-w-[300px]">
                            <GameCard game={game} onClick={() => navigate(`/game/${gameId}`)} />
                        </div>
                    ))}
                </div>
            </div>
            <div className="flex justify-between items-center mt-2">
                <button
                    onClick={scrollPrev}
                    className="px-3 py-1 bg-gray-300 hover:bg-gray-400 rounded disabled:opacity-50"
                    disabled={ind == 0}
                >
                    &lt; Prev
                </button>
                <button
                    onClick={scrollNext}
                    className="px-3 py-1 bg-gray-300 hover:bg-gray-400 rounded disabled:opacity-50"
                    disabled={!(page === lastPage) && ind == games.length-1}
                >
                    Next &gt;
                </button>
            </div>
        </div>
    );
};

export default GameCarousel;

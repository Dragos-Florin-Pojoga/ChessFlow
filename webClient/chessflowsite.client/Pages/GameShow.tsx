import React, { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/NavBar.js";
import { getToken } from "../Utils/authToken.ts";
import UserStore from '../stores/UserStore.ts';
import { Chessboard } from "react-chessboard";

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

function GameShow() {
    const [searchParams, setSearchParams] = useSearchParams();

    const initialUsernameOne = searchParams.get("usernameOne") || '';
    const initialUsernameTwo = searchParams.get("usernameTwo") || '';
    const initialSortType = searchParams.get("sortType") || 'id';
    const initialIsAscending = searchParams.get("isAscending") === 'true';

    const [games, setGames] = useState<GameDataEntry[]>([]);
    const [usernameOne, setUsernameOne] = useState(initialUsernameOne);
    const [usernameTwo, setUsernameTwo] = useState(initialUsernameTwo);
    const [sortType, setSortType] = useState(initialSortType);
    const [isAscending, setIsAscending] = useState<boolean>(initialIsAscending);
    const [page, setPage] = useState(1);
    const [pageSize] = useState(10);
    const [lastPage, setlastPage] = useState(0);

    //temp values for storing inputed but not submitted fields when changing page
    const [tempUsernameOne, setTempUsernameOne] = useState(initialUsernameOne);
    const [tempUsernameTwo, setTempUsernameTwo] = useState(initialUsernameTwo);
    const [tempSortType, setTempSortType] = useState(initialSortType);
    const [tempIsAscending, setTempIsAscending] = useState<boolean>(initialIsAscending);

    const [nonpag, setNonpag] = useState<boolean>(false);

    // state variable for error messages (and also other messages)
    const [errors, setErrors] = useState<string[]>([]);

    const navigate = useNavigate();

    const { user, setUser } = UserStore();

    const token = getToken();

    const setError = (e: string) => setErrors([e]);

    const fetchGames = async () => {
        const params = new URLSearchParams({
            page: "" + page,
            pageSize: "" + pageSize,
            usernameOne,
            usernameTwo,
            sortType,
            isAscending: isAscending.toString()
        });

        console.log(params.toString());

        const response = await fetch(`/api/game/index?${params.toString()}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`
            }
        });

        console.log(response);

        if (response.ok) {
            const data = await response.json();
            console.log(data);
            setGames(data.items);
            setlastPage(data.lastPage);
        } else {
            console.error('Failed to fetch games');
        }
    };

    useEffect(() => {
        setPage(1);
        setNonpag(true);
        fetchGames();
    }, [usernameOne, usernameTwo, sortType, isAscending]);

    useEffect(() => {
        if (!nonpag) fetchGames();
        setNonpag(false);
    }, [page]);

    const handleSearch = (e) => {
        e.preventDefault();
        setUsernameOne(tempUsernameOne);
        setUsernameTwo(tempUsernameTwo);
        setSortType(tempSortType);
        setIsAscending(tempIsAscending);
    };

    const nameContent = (game: GameData, side: "white" | "black") => {
        const username = side === "white" ? game.whiteName : game.blackName;
        const isGuest = side === "white" ? game.isWhiteGuest : game.isBlackGuest;
        const elo = side === "white" ? game.whiteElo : game.blackElo;
        const deltaElo = side === "white" ? game.whiteDeltaElo : game.blackDeltaElo;

        const deltaEloContent = deltaElo === null ? (
            <></>
        ) : (
            <span className={deltaElo > 0 ? "plus-elo" : "minus-elo"}>
                ({deltaElo > 0 ? `+${deltaElo}` : deltaElo})
            </span>
        );

        const usernameContent = isGuest ? (
            <span className="text-gray-500">{username} (guest)</span>
        ) : (
                <a className={"unstyled"} href="#" onClick={() => navigate(`/user/${username}`)}>{username}</a>
        );
        return (<span>{usernameContent} {elo}{deltaEloContent}</span>)
    }

    const getChessboard = (game: GameData) => {
        if (game.fen === null) {
            return <div className="warning">Couldn't fetch board</div>
        }
        return (<Chessboard
            position={game.fen}
            arePiecesDraggable={false}
            boardWidth={100}
        ></Chessboard>);
    }

    return (
        <AuthorizeView>
            <NavBar></NavBar>
            <div className="tailwind-page">
                <div className="p-4">
                    <h2 className="text-2xl font-bold mb-4">Games</h2>

                    <form onSubmit={handleSearch} className="flex gap-4 mb-4">
                        <input
                            type="text"
                            placeholder="Filter by first username"
                            value={tempUsernameOne}
                            onChange={(e) => setTempUsernameOne(e.target.value)}
                            className="border px-2 py-1 rounded"
                        />
                        <input
                            type="text"
                            placeholder="Filter by second username"
                            value={tempUsernameTwo}
                            onChange={(e) => setTempUsernameTwo(e.target.value)}
                            className="border px-2 py-1 rounded"
                        />
                        <select
                            value={tempSortType}
                            onChange={(e) => setTempSortType(e.target.value)}
                            className="border px-2 py-1 rounded"
                        >
                            <option value="id">ID</option>
                            <option value="date">Time started</option>
                            <option value="elodif">Elo change difference</option>
                        </select>
                        <select
                            value={tempIsAscending ? 'asc' : 'desc'}
                            onChange={(e) => setTempIsAscending(e.target.value === 'asc')}
                            className="border px-2 py-1 rounded"
                        >
                            <option value="asc">Asc</option>
                            <option value="desc">Desc</option>
                        </select>
                        <button type="submit" className="bg-blue-500 text-white px-4 py-1 rounded">
                            Search
                        </button>
                    </form>

                    <table className="w-full border border-gray-300 mb-4">
                        <thead>
                            <tr className="bg-gray-100">
                                <th className="border px-2 py-1">ID</th>
                                <th className="border px-2 py-1">White player</th>
                                <th className="border px-2 py-1">Black player</th>
                                <th className="border px-2 py-1">Format</th>
                                <th className="border px-2 py-1">Result</th>
                                <th className="border px-2 py-1">Time started</th>
                                <th className="border px-2 py-1"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {games.length === 0 ? (
                                <tr>
                                    <td colSpan={5} className="text-center py-4 text-gray-500">
                                        No reports found.
                                    </td>
                                </tr>
                            ) : (
                                games.map((r) => (
                                    <tr key={r.gameId}>
                                        <td className="border px-2 py-1">{r.gameId}</td>
                                        <td className="border px-2 py-1">{nameContent(r.game, "white")}</td>
                                        <td className="border px-2 py-1">{nameContent(r.game, "black")}</td>
                                        <td className="border px-2 py-1">{r.game.format}</td>
                                        <td className="border px-2 py-1">{r.game.result}</td>
                                        <td className="border px-2 py-1">{r.game.startTime}</td>
                                        <td className="border px-2 py-1">
                                            <div onClick={() => navigate(`/game/${r.gameId}`) }>{getChessboard(r.game)}</div>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>

                    </table>

                    {
                        games.length === 0 ?
                            <></>
                            :
                            <div className="flex justify-between items-center">
                                <button
                                    disabled={page === 1}
                                    onClick={() => setPage((p) => p - 1)}
                                    className="bg-gray-200 px-3 py-1 rounded disabled:opacity-50"
                                >
                                    Prev
                                </button>
                                <span>Page {page} of {lastPage}</span>
                                <button
                                    disabled={page >= lastPage}
                                    onClick={() => setPage((p) => p + 1)}
                                    className="bg-gray-200 px-3 py-1 rounded disabled:opacity-50"
                                >
                                    Next
                                </button>
                            </div>
                    }
                </div>
            </div>

        </AuthorizeView >
    );
}


export default GameShow;
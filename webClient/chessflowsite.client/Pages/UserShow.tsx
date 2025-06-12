import React, { useState, useEffect } from 'react';
import { useNavigate, Navigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/Navbar.tsx";
import { getToken } from "../Utils/authToken.ts";
import RequireRole from '../Components/RequireRole.js';
import UnbanLink from '../Components/UnbanLink.tsx';

import '../src/TailwindScoped.css';

function UserShow() {
    const [users, setUsers] = useState([]);
    const [sortType, setSortType] = useState('elo');
    const [isAscending, setIsAscending] = useState<boolean>(false);
    const [page, setPage] = useState(1);
    const [pageSize] = useState(10);
    const [lastPage, setlastPage] = useState(0);

    //temp values for storing inputed but not submitted fields when changing page
    const [tempSortType, setTempSortType] = useState('id');
    const [tempIsAscending, setTempIsAscending] = useState<boolean>(false);

    const [nonpag, setNonpag] = useState<boolean>(false);

    // state variable for error messages (and also other messages)
    const [errors, setErrors] = useState<string[]>([]);

    const navigate = useNavigate();

    const token = getToken();

    const setError = (e: string) => setErrors([e]);

    const fetchReports = async () => {
        const params = new URLSearchParams({
            page: "" + page,
            pageSize: "" + pageSize,
            sortType,
            isAscending: isAscending.toString()
        });

        console.log(params.toString());

        const response = await fetch(`/api/account/index?${params.toString()}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`
            }
        });

        console.log(response);

        if (response.ok) {
            const data = await response.json();
            console.log(data);
            setUsers(data.items);
            setlastPage(data.lastPage);
        } else {
            console.error('Failed to fetch reports');
        }
    };

    useEffect(() => {
        setPage(1);
        setNonpag(true);
        fetchReports();
    }, [sortType, isAscending]);

    useEffect(() => {
        if (!nonpag) fetchReports();
        setNonpag(false);
    }, [page]);

    const handleSearch = (e) => {
        e.preventDefault();
        setSortType(tempSortType);
        setIsAscending(tempIsAscending);
    };

    const handleUnbanned = (name: string) => {
        setUsers((users) =>
            users.map((user) =>
                user.name === name ? { ...user, banned: false } : user
            )
        );
    };

    return (
        <AuthorizeView>
            <NavBar></NavBar>
            <div className="tailwind-page">
                <div className="p-4">
                    <h2 className="text-2xl font-bold mb-4">Users</h2>
                    <form onSubmit={handleSearch} className="flex gap-4 mb-4">
                        <select
                            value={tempSortType}
                            onChange={(e) => setTempSortType(e.target.value)}
                            className="border px-2 py-1 rounded"
                        >
                            <option value="elo">Elo</option>
                            <option value="games">Number of games</option>
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
                                <th className="border px-2 py-1">Username</th>
                                <th className="border px-2 py-1">Elo</th>
                                <th className="border px-2 py-1">Number of games</th>
                                <th className="border px-2 py-1"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {users.length === 0 ? (
                                <tr>
                                    <td colSpan={5} className="text-center py-4 text-gray-500">
                                        No users found.
                                    </td>
                                </tr>
                            ) : (
                                users.map((r) => (
                                    <tr key={r.name}>
                                        <td className="border px-2 py-1">{r.isAdmin ? <span className="message">[ADMIN]</span> : <></>}<a className={"unstyled"} href="#" onClick={() => navigate(`/user/${r.name}`)}>{r.name}</a></td>
                                        <td className="border px-2 py-1">{r.elo}</td>
                                        <td className="border px-2 py-1">{r.games}</td>
                                        <td className="border px-2 py-1">
                                            {
                                                r.banned ?
                                                    <>
                                                        <span className={"warning"}>User is banned! </span>
                                                        <RequireRole roles={["Admin"]} link={true}><UnbanLink onUnbanned={() => handleUnbanned(r.name)} username={r.name}>Unban user</UnbanLink></RequireRole>
                                                    </>
                                                    :
                                                    <RequireRole roles={["Admin"]} link={true}><span><a href="#" onClick={() => navigate(`/ban/${r.name}`)}>Create ban</a></span></RequireRole>
                                            }
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>

                    </table>

                    {
                        users.length === 0 ?
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
        </AuthorizeView>
    );
}


export default UserShow;
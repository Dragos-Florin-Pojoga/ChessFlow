import React, { useState, useEffect } from 'react';
import { useNavigate, Navigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/NavBar.js";
import { getToken } from "../Utils/authToken.ts";
import UserStore from '../stores/UserStore.ts';
import RequireRole from '../Components/RequireRole.js';

import '../src/TailwindScoped.css';

function BanShow() {
    const [bans, setBans] = useState([]);
    const [bannedName, setBannedName] = useState('');
    const [sortType, setSortType] = useState('id');
    const [isAscending, setIsAscending] = useState<boolean>(false);
    const [showActiveOnly, setShowActiveOnly] = useState<boolean>(false);
    const [page, setPage] = useState(1);
    const [pageSize] = useState(10);
    const [lastPage, setlastPage] = useState(0);

    //temp values for storing inputed but not submitted fields when changing page
    const [tempBannedName, setTempBannedName] = useState('');
    const [tempSortType, setTempSortType] = useState('id');
    const [tempIsAscending, setTempIsAscending] = useState<boolean>(false);
    const [tempShowActiveOnly, setTempShowActiveOnly] = useState<boolean>(false);

    const [nonpag, setNonpag] = useState<boolean>(false);

    // state variable for error messages (and also other messages)
    const [errors, setErrors] = useState<string[]>([]);

    const navigate = useNavigate();

    const { user, setUser } = UserStore();

    const token = getToken();

    const setError = (e: string) => setErrors([e]);

    const fetchReports = async () => {
        const params = new URLSearchParams({
            page: "" + page,
            pageSize: "" + pageSize,
            bannedName,
            sortType,
            isAscending: isAscending.toString(),
            showActiveOnly: showActiveOnly.toString()
        });

        console.log(params.toString());

        const response = await fetch(`/api/bans/index?${params.toString()}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`
            }
        });

        console.log(response);

        if (response.ok) {
            const data = await response.json();
            console.log(data);
            setBans(data.items);
            setlastPage(data.lastPage);
        } else {
            console.error('Failed to fetch reports');
        }
    };

    useEffect(() => {
        setPage(1);
        setNonpag(true);
        fetchReports();
    }, [bannedName, sortType, isAscending, showActiveOnly]);

    useEffect(() => {
        if (!nonpag) fetchReports();
        setNonpag(false);
    }, [page]);

    const handleSearch = (e) => {
        e.preventDefault();
        setBannedName(tempBannedName);
        setSortType(tempSortType);
        setIsAscending(tempIsAscending);
        setShowActiveOnly(tempShowActiveOnly);
    };

    return (
        <AuthorizeView>
            <RequireRole roles={["Admin"]}>
                <NavBar></NavBar>
                <div className="tailwind-page">
                    <div className="p-4">
                        <h2 className="text-2xl font-bold mb-4">Bans</h2>

                        <form onSubmit={handleSearch} className="flex gap-4 mb-4">
                            <input
                                type="text"
                                placeholder="Filter by banned user"
                                value={tempBannedName}
                                onChange={(e) => setTempBannedName(e.target.value)}
                                className="border px-2 py-1 rounded"
                            />
                            <select
                                value={tempSortType}
                                onChange={(e) => setTempSortType(e.target.value)}
                                className="border px-2 py-1 rounded"
                            >
                                <option value="id">ID</option>
                                <option value="banStart">Ban start</option>
                            </select>
                            <select
                                value={tempIsAscending ? 'asc' : 'desc'}
                                onChange={(e) => setTempIsAscending(e.target.value === 'asc')}
                                className="border px-2 py-1 rounded"
                            >
                                <option value="asc">Asc</option>
                                <option value="desc">Desc</option>
                            </select>
                            <label className="flex items-center gap-1">
                                <input
                                    type="checkbox"
                                    checked={tempShowActiveOnly}
                                    onChange={(e) => setTempShowActiveOnly(e.target.checked)}
                                />
                                Only show active
                            </label>
                            <button type="submit" className="bg-blue-500 text-white px-4 py-1 rounded">
                                Search
                            </button>
                        </form>

                        <table className="w-full border border-gray-300 mb-4">
                            <thead>
                                <tr className="bg-gray-100">
                                    <th className="border px-2 py-1">ID</th>
                                    <th className="border px-2 py-1">Banned User</th>
                                    <th className="border px-2 py-1">Issuer</th>
                                    <th className="border px-2 py-1">Report ID</th>
                                    <th className="border px-2 py-1">Reason</th>
                                    <th className="border px-2 py-1">Ban start</th>
                                    <th className="border px-2 py-1">Ban end</th>
                                    <th className="border px-2 py-1">Status</th>
                                </tr>
                            </thead>
                            <tbody>
                                {bans.length === 0 ? (
                                    <tr>
                                        <td colSpan={5} className="text-center py-4 text-gray-500">
                                            No bans found.
                                        </td>
                                    </tr>
                                ) : (
                                    bans.map((r) => (
                                        <tr key={r.banID}>
                                            <td className="border px-2 py-1">{r.banID}</td>
                                            <td className="border px-2 py-1">{r.bannedName}</td>
                                            <td className="border px-2 py-1">{r.issuerName}</td>
                                            <td className="border px-2 py-1">{r.reportID ?? 'N/A'}</td>
                                            <td className="border px-2 py-1">{r.reason}</td>
                                            <td className="border px-2 py-1">{r.start}</td>
                                            <td className="border px-2 py-1">{r.end ?? 'Permanent'}</td>
                                            <td className="border px-2 py-1">
                                                {
                                                    (r.end === null || new Date(r.end) > new Date()) ?
                                                        <span>Active</span>
                                                        :
                                                        <span>Expired</span>
                                                }
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>

                        </table>

                        {
                            bans.length === 0 ?
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

            </RequireRole>
        </AuthorizeView >
    );
}


export default BanShow;
import React, { useState, useEffect } from 'react';
import {useNavigate, Navigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/Navbar.tsx";
import { getToken } from "../Utils/authToken.ts";
import UserStore from '../Stores/UserStore.ts';
import RequireRole from '../Components/RequireRole.js';
import UnbanLink from '../Components/UnbanLink.tsx';

import '../src/TailwindScoped.css';

function ReportShow() {
    const [reports, setReports] = useState([]);
    const [reportedName, setReportedName] = useState('');
    const [sortType, setSortType] = useState('id');
    const [isAscending, setIsAscending] = useState<boolean>(false);
    const [page, setPage] = useState(1);
    const [pageSize] = useState(10);
    const [lastPage, setlastPage] = useState(0);

    //temp values for storing inputed but not submitted fields when changing page
    const [tempReportedName, setTempReportedName] = useState('');
    const [tempSortType, setTempSortType] = useState('id');
    const [tempIsAscending, setTempIsAscending] = useState<boolean>(false);

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
            reportedName,
            sortType,
            isAscending: isAscending.toString()
        });

        console.log(params.toString());

        const response = await fetch(`/api/reports/index?${params.toString()}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`
            }
        });

        console.log(response);

        if (response.ok) {
            const data = await response.json();
            console.log(data);
            setReports(data.items);
            setlastPage(data.lastPage);
        } else {
            console.error('Failed to fetch reports');
        }
    };

    useEffect(() => {
        setPage(1);
        setNonpag(true);
        fetchReports();
    }, [reportedName, sortType, isAscending]);

    useEffect(() => {
        if (!nonpag) fetchReports();
        setNonpag(false);
    }, [page]);

    const handleSearch = (e) => {
        e.preventDefault();
        setReportedName(tempReportedName);
        setSortType(tempSortType);
        setIsAscending(tempIsAscending);
    };

    const handleUnbanned = (name: string) => {
        setReports((reports) =>
            reports.map((report) =>
                report.reportedName === name ? { ...report, reportedBanned: false } : report
            )
        );
    };

    return (
        <AuthorizeView>
            <RequireRole roles={["Admin"]}>
                <NavBar></NavBar>
                <div className="tailwind-page">
                    <div className="p-4">
                        <h2 className="text-2xl font-bold mb-4">Reports</h2>

                        <form onSubmit={handleSearch} className="flex gap-4 mb-4">
                            <input
                                type="text"
                                placeholder="Filter by reported user"
                                value={tempReportedName}
                                onChange={(e) => setTempReportedName(e.target.value)}
                                className="border px-2 py-1 rounded"
                            />
                            <select
                                value={tempSortType}
                                onChange={(e) => setTempSortType(e.target.value)}
                                className="border px-2 py-1 rounded"
                            >
                                <option value="id">ID</option>
                                <option value="date">Date created</option>
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
                                    <th className="border px-2 py-1">Reported</th>
                                    <th className="border px-2 py-1">Reportee</th>
                                    <th className="border px-2 py-1">Game ID</th>
                                    <th className="border px-2 py-1">Reason</th>
                                    <th className="border px-2 py-1">Date created</th>
                                    <th className="border px-2 py-1"></th>
                                    <th className="border px-2 py-1"></th>
                                </tr>
                            </thead>
                            <tbody>
                                {reports.length === 0 ? (
                                    <tr>
                                        <td colSpan={5} className="text-center py-4 text-gray-500">
                                            No reports found.
                                        </td>
                                    </tr>
                                ) : (
                                    reports.map((r) => (
                                        <tr key={r.reportID}>
                                            <td className="border px-2 py-1">{r.reportID}</td>
                                            <td className="border px-2 py-1">{r.reportedName}</td>
                                            <td className="border px-2 py-1">{r.reporteeName}</td>
                                            <td className="border px-2 py-1">{r.gameID != null ? (<a href="#" onClick={() => navigate(`/game/${r.gameID}`)}>{r.gameID}</a>) : 'N/A'}</td>
                                            <td className="border px-2 py-1">{r.reason}</td>
                                            <td className="border px-2 py-1">{r.created}</td>
                                            <td className="border px-2 py-1">
                                                {
                                                    r.reportedBanned ? <></>
                                                        :
                                                        <a href="#" onClick={() => navigate(`/games/?usernameOne=${r.reportedName}`)}>See all games</a>
                                                }
                                            </td>
                                            <td className="border px-2 py-1">
                                                {
                                                    r.reportedBanned ?
                                                        <>
                                                            <span className={"warning"}>User is banned! </span>
                                                            <RequireRole roles={["Admin"]} link={true}><UnbanLink onUnbanned={() => handleUnbanned(r.reportedName)} username={r.reportedName}>Unban user</UnbanLink></RequireRole>
                                                        </>
                                                        :
                                                        <RequireRole roles={["Admin"]} link={true}><span><a href="#" onClick={() => navigate(`/ban/${r.reportedName}?reportID=${r.reportID}`)}>Create ban</a></span></RequireRole>
                                                }
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>

                        </table>

                        {
                            reports.length === 0 ?
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


export default ReportShow;
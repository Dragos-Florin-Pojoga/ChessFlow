import './App.css';

import React, { useEffect } from "react";
import { BrowserRouter, Routes, Route } from 'react-router-dom';

import useSignalRStore from '../Stores/SignalRStore.ts';

import Home from '../Pages/Home.tsx';
import Login from '../Pages/Login.tsx';
import Register from '../Pages/Register.tsx';
import AdminPanel from '../Pages/AdminPanel.tsx';
import Unauthorized from '../Pages/Unauthorized.tsx';
import UserInfo from '../Pages/UserInfo.tsx';
import Report from '../Pages/Report.tsx';
import Ban from '../Pages/Ban.tsx';
import ReportShow from '../Pages/ReportShow.tsx';
import UserShow from '../Pages/UserShow.tsx';
import BanShow from '../Pages/BanShow.tsx';
import GameHub from '../Pages/GameHub.tsx';
import ChessGame from '../Pages/ChessGame.tsx';
import GameInfo from '../Pages/GameInfo.tsx';
import GameShow from '../Pages/GameShow.tsx';
function App() {

    useEffect(() => {
        useSignalRStore.getState().startConnection();
    }, []);

    return (
        <BrowserRouter>
            <Routes>
                <Route path="/login" element={<Login />} />
                <Route path="/register" element={<Register />} />
                <Route path="/unauthorized" element={<Unauthorized/> } />
                <Route path="/admin" element={<AdminPanel />} />
                <Route path="/admin/reports" element={<ReportShow />} />
                <Route path="/admin/bans" element={<BanShow />} />
                <Route path="/users" element={<UserShow />} />
                <Route path="/user/:username" element={<UserInfo />} />
                <Route path="/report/:username" element={<Report />} />
                <Route path="/ban/:username" element={<Ban />} />
                <Route path="/gamehub" element={<GameHub />} />"
                <Route path="/game/play/:id" element={<ChessGame />} />"
                <Route path="/game/:id" element={<GameInfo />} />"
                <Route path="/games" element={<GameShow />} />
                <Route path="/" element={<Home />} />
            </Routes>
        </BrowserRouter>
    );

}

export default App;
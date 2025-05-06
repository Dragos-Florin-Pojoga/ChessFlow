import './App.css';

import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Home from '../Pages/Home.tsx';
import Login from '../Pages/Login.tsx';
import Register from '../Pages/Register.tsx';
import AdminPanel from '../Pages/AdminPanel.tsx';
import Unauthorized from '../Pages/Unauthorized.tsx';
function App() {


    return (
        <BrowserRouter>
            <Routes>
                <Route path="/login" element={<Login />} />
                <Route path="/register" element={<Register />} />
                <Route path="/unauthorized" element={<Unauthorized/> } />
                <Route path="/admin" element={<AdminPanel />} />
                <Route path="/" element={<Home />} />
            </Routes>
        </BrowserRouter>
    );

}

export default App;
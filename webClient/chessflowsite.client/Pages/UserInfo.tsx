import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/NavBar.js";
import { getToken } from "../Utils/authToken.ts";
import UserStore from '../stores/UserStore.ts';
import GameCarousel from '../Components/GameCarousel.tsx';
import GameShow from './GameShow.tsx';

function UserInfo() {
    const param = useParams();
    const [exists, setExists] = useState(false);
    const [elo, setElo] = useState(1200);
    const [name1, setName1] = useState("");
    const [banned, setBanned] = useState<boolean>(false);

    const navigate = useNavigate();

    const { user, setUser } = UserStore();

    const username = param.username;

    const token = getToken();

    const handleReportClick = () => {
        navigate(`/report/${username}`);
    }

    useEffect(() => {

        fetch(`/api/account/user/${username}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            }
        })
            .then(async (response) => {
                if (response.ok) {
                    let data = await response.json();
                    setName1(data.name);
                    setElo(data.elo);
                    setExists(true);
                    setBanned(data.banned);
                }
            });
    }, [username]);

    return (
        <AuthorizeView>
            <NavBar></NavBar>
            {exists ?
                <>
                    <h2>Username: {name1}</h2>
                    <h2>Elo: {elo}</h2>
                    <br></br>
                    <div>
                        {
                            banned ?
                                <div className="warning">User is banned!</div>
                            :
                            (user.name === name1) ?
                                <></>
                                :
                                <button type="button" onClick={handleReportClick}>Report!</button>
                        }
                    </div>
                    <h2>Recent games: </h2>
                    <GameCarousel username={name1} />
                    <div><a href="#" onClick={() => navigate(`/games/?usernameOne=${name1}`)}>See games as list</a></div>
                </>
                :
                <h1>User doesn't exist!</h1>
            }
        </AuthorizeView >
    );
}



export default UserInfo;
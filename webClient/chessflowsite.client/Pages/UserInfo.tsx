import React, { useState, useEffect } from 'react';
import { useParams } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/NavBar.js";
import { getToken } from "../Utils/authToken.ts";


function UserInfo() {
    const param = useParams();
    const [exists, setExists] = useState(false);
    const [elo, setElo] = useState(1200);
    const [name1, setName1] = useState("");

    console.log(param);
    const username = param.username;

    const token = getToken();

    console.log(token);

    useEffect(() => {

        fetch(`/api/account/user/${username}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            }
        })
            .then(async (response) => {
                console.log(response);
                if (response.ok) {
                    let data = await response.json();
                    console.log(data);
                    setName1(data.name);
                    setElo(data.elo);
                    setExists(true);
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
                </>
                :
                <h1>User doesn't exist!</h1>}
        </AuthorizeView>
    );
}



export default UserInfo;
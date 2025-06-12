
import { useNavigate } from "react-router-dom";
import { getToken, clearToken } from "../Utils/authToken.ts";
import SignalRStore from "../Stores/SignalRStore.ts";
import GameStore from '../Stores/GameStore.js';

function LogoutLink(props: { children: React.ReactNode }) {

    const navigate = useNavigate();

    const { startConnection, stopConnection } = SignalRStore();

    const { clearGame } = GameStore();

    const token = getToken();


    const handleSubmit = (e: React.FormEvent<HTMLAnchorElement>) => {
        e.preventDefault();
        fetch("/api/account/logout", {
            method: "POST",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            },
            body: ""

        })
            .then((data) => {
                if (data.ok) {

                    navigate("/");
                }


            })
            .catch((error) => {
                console.error(error);
            })

        clearToken();
        stopConnection();  // kill authenticated connection
        startConnection();
        setTimeout(() => clearGame(), 100);
    };

    return (
        <>
            <a href="#" onClick={handleSubmit}>{props.children}</a>
        </>
    );
}

export default LogoutLink;
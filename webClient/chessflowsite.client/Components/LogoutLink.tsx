
import { useNavigate } from "react-router-dom";
import { getToken, clearToken } from "../Utils/authToken.ts";
import SignalRStore from "../Stores/SignalRStore.ts";

function LogoutLink(props: { children: React.ReactNode }) {

    const navigate = useNavigate();

    const { startConnection, stopConnection } = SignalRStore();

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
    };

    return (
        <>
            <a href="#" onClick={handleSubmit}>{props.children}</a>
        </>
    );
}

export default LogoutLink;
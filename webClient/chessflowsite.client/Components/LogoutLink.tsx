
import { useNavigate } from "react-router-dom";

function LogoutLink(props: { children: React.ReactNode }) {

    const navigate = useNavigate();

    function getToken(): string | null {
        return localStorage.getItem("token") || sessionStorage.getItem("token");
    }

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

        localStorage.removeItem("token");
        sessionStorage.removeItem("token");
    };

    return (
        <>
            <a href="#" onClick={handleSubmit}>{props.children}</a>
        </>
    );
}

export default LogoutLink;

import { useNavigate } from "react-router-dom";
import LogoutLink from "../Components/LogoutLink.tsx";
import RequireRole from "../Components/RequireRole.tsx";
import { AuthorizedUser } from "../Components/AuthorizeView.tsx";
import OwnProfileLink from "../Components/OwnProfileLink.tsx";
import { getToken } from "../Utils/authToken.ts";
import '../src/App.css';

function NavBar(props: { children: React.ReactNode }) {

    const navigate = useNavigate();

    const token = getToken();

    return (
            <div className="topnav">
            {token ?
                <span><LogoutLink>Logout <AuthorizedUser value="email" /></LogoutLink></span>
                :
                <span><a href="#" onClick={() => navigate("/login")}>Log in</a></span>}
            <RequireRole roles={["Admin"]} link={true}><a href="#" onClick={() => navigate("/admin")}>Admin panel</a></RequireRole>
            <span><OwnProfileLink>Go to profile</OwnProfileLink></span>
            <span><a href="#" onClick={() => navigate("/users")}>Users stats</a></span>
            <span><a href="#" onClick={() => navigate("/")}>Go Home</a></span>
            </div>
    );
}

export default NavBar;
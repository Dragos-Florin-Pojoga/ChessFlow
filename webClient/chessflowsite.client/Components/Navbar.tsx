
import { useNavigate, Link } from "react-router-dom";
import LogoutLink from "../Components/LogoutLink.tsx";
import RequireRole from "../Components/RequireRole.tsx";
import { AuthorizedUser } from "../Components/AuthorizeView.tsx";
import OwnProfileLink from "../Components/OwnProfileLink.tsx";
import { getToken } from "../Utils/authToken.ts";
import GameStore from '../Stores/GameStore.js';
import '../src/App.css';

function NavBar(props: { children: React.ReactNode }) {

    const navigate = useNavigate();

    const { game } = GameStore();

    const token = getToken();

    return (
        <div className="topnav">
            {token ?
                <span><LogoutLink>Logout <AuthorizedUser value="email" /></LogoutLink></span>
                :
                <span><Link to="/login">Log in</Link></span>}
            <RequireRole roles={["Admin"]} link={true}><Link to="/admin">Admin panel</Link></RequireRole>
            <span><OwnProfileLink>Go to profile</OwnProfileLink></span>
            <span><Link to="/users">Users stats</Link></span>
            <span><Link to="/">Go Home</Link></span>
            <span><Link to="/games">Games</Link></span>
            {
                (game != null && game.isOver == false) ? 
                    <span><Link to={`/game/play/${game.id}`}>Return to game</Link></span>
                    :
                    <></>
            }
        </div>
    );
}

export default NavBar;
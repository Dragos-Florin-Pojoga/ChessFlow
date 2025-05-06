
import { useNavigate } from "react-router-dom";
import LogoutLink from "../Components/LogoutLink.tsx";
import RequireRole from "../Components/RequireRole.tsx";
import { AuthorizedUser } from "../Components/AuthorizeView.tsx";
import OwnProfileLink from "../Components/OwnProfileLink.tsx";
import '../src/App.css';

function NavBar(props: { children: React.ReactNode }) {

    const navigate = useNavigate();

    function getToken(): string | null {
        return localStorage.getItem("token") || sessionStorage.getItem("token");
    }

    const token = getToken();

    const handleLogSubmit = () => {
        navigate("/login");
    };

    const handleAdminSubmit = () => {
        navigate("/admin");
    };

    const handleHomeSubmit = () => {
        navigate("/");
    };

    return (
            <div className="topnav">
            {token ?
                <span><LogoutLink>Logout <AuthorizedUser value="email" /></LogoutLink></span>
                :
                <span><a href="#" onClick={handleLogSubmit}>Log in</a></span>}
            <RequireRole roles={["Admin"]} link={true}><a href="#" onClick={handleAdminSubmit}>Admin panel</a></RequireRole>
            <span><OwnProfileLink>Go to profile</OwnProfileLink></span>
            <span><a href="#" onClick={handleHomeSubmit}>Go Home</a></span>
            </div>
    );
}

export default NavBar;

import { useNavigate } from "react-router-dom";
import LogoutLink from "../Components/LogoutLink.tsx";
import RequireRole from "../Components/RequireRole.tsx";
import { AuthorizedUser } from "../Components/AuthorizeView.tsx";
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

    return (
            <div className="topnav">
            {token ?
                <span><LogoutLink>Logout <AuthorizedUser value="email" /></LogoutLink></span>
                :
                <a href="#" onClick={handleLogSubmit}>Log in</a>}
            <RequireRole roles={["Admin"]} link={true}><a href="#" onClick={handleAdminSubmit}>Admin panel</a></RequireRole>
            </div>
    );
}

export default NavBar;
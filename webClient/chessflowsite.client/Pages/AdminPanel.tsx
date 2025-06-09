import { useNavigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.tsx";
import NavBar from "../Components/Navbar.tsx";
import RequireRole from "../Components/RequireRole.tsx";

function AdminPanel() {

    const navigate = useNavigate();

    const handleReportSubmit = () => {
        navigate("/admin/reports");
    };
    const handleBanSubmit = () => {
        navigate("/admin/bans");
    };

    return (
        <AuthorizeView>
            <RequireRole roles={["Admin"]}>
                <NavBar></NavBar>
                <h1 id="tabelLabel">Admin Panel</h1>
                <div><a href="#" onClick={handleReportSubmit}>Go to reports page</a></div>
                <div><a href="#" onClick={handleBanSubmit}>Go to bans page</a></div>
            </RequireRole>
            
        </AuthorizeView>
    );
}



export default AdminPanel;
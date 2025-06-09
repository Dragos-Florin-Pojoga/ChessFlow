import { useNavigate, Link } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.tsx";
import NavBar from "../Components/NavBar.tsx";
import RequireRole from "../Components/RequireRole.tsx";

function AdminPanel() {

    const navigate = useNavigate();

    return (
        <AuthorizeView>
            <RequireRole roles={["Admin"]}>
                <NavBar></NavBar>
                <h1 id="tabelLabel">Admin Panel</h1>
                <div><Link to="/admin/reports">Go to reports page</Link></div>
                <div><Link to="/admin/bans">Go to bans page</Link></div>
            </RequireRole>
            
        </AuthorizeView>
    );
}



export default AdminPanel;
import AuthorizeView from "../Components/AuthorizeView.tsx";
import NavBar from "../Components/NavBar.tsx";
import RequireRole from "../Components/RequireRole.tsx";


function AdminPanel() {
    return (
        <AuthorizeView>
            <RequireRole roles={["Admin"]}>
                <NavBar></NavBar>
                <h1 id="tabelLabel">Admin Panel</h1>
            </RequireRole>
            
        </AuthorizeView>
    );
}



export default AdminPanel;
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/NavBar.js";
import { useNavigate } from "react-router-dom";


function Unauthorized() {
    const navigate = useNavigate();

    const handleClick = () => {
        navigate('/');
    };

    return (
        <AuthorizeView>
            <NavBar></NavBar>
            <h1 id="tabelLabel">You are not authorized to acces this page!</h1>
            <button type="button" onClick={handleClick}>Go back to homepage</button>
        </AuthorizeView>
    );
}



export default Unauthorized;
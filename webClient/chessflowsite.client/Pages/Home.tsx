import AuthorizeView, { AuthorizedUser } from "../Components/AuthorizeView.tsx";
import NavBar from "../Components/Navbar.tsx";
import { useNavigate } from "react-router-dom";


function Home() {
    const navigate = useNavigate();

    return (
        <>
            <NavBar></NavBar>
            <h1 id="tabelLabel">ChessFlow</h1>
            <button onClick={() => navigate("/gamehub") }>Play game</button>

        </>
    );
}

/*
<AuthorizeView>
            <span><LogoutLink>Logout <AuthorizedUser value="email" /></LogoutLink></span>
            <WeatherForecast />
</AuthorizeView>
*/

export default Home;
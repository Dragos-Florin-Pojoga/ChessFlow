import AuthorizeView, { AuthorizedUser } from "../Components/AuthorizeView.tsx";
import NavBar from "../Components/NavBar.tsx";


function Home() {
    return (
        <>
            <NavBar></NavBar>
            <h1 id="tabelLabel">ChessFlow</h1>
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
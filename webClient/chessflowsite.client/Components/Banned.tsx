import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/Navbar.tsx";
import { useNavigate } from "react-router-dom";


function Banned(props: { children: React.ReactNode }) {
    const navigate = useNavigate();

    const handleClick = () => {
        navigate('/');
    };

    return (
        <>
            <NavBar></NavBar>
            <h1 id="tabelLabel">{props.children}</h1>
            <button type="button" onClick={handleClick}>Go back to homepage</button>
        </>
    );
}



export default Banned;
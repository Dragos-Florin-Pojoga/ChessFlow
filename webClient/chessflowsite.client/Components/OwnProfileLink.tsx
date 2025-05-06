
import { useNavigate } from "react-router-dom";
import UserStore from '../stores/UserStore.ts';
import { getToken } from "../Utils/authToken.ts";

function OwnProfileLink(props: { children: React.ReactNode }) {

    const navigate = useNavigate();

    const token = getToken();

    const { user, userStore } = UserStore();

    const handleSubmit = (e: React.FormEvent<HTMLAnchorElement>) => {
        e.preventDefault();
        navigate(`/user/${user.name}`);
    }

    return (
        <>
            {
                token ?
                    <>
                        < a href="#" onClick={handleSubmit} > {props.children}</a >
                    </>
                    :
                    <></>
            }
        </>
    );
}

export default OwnProfileLink;
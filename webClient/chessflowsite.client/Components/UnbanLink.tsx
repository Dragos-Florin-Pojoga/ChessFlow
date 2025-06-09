import React, { useState } from 'react';
import { useNavigate } from "react-router-dom";
import { getToken } from "../Utils/authToken.ts";
import UserStore from '../Stores/UserStore.ts';
interface UnbanLinkProps {
    username: string;
    children: React.ReactNode;
    onUnbanned?: () => void;
}
function UnbanLink({ username, onUnbanned, children }: UnbanLinkProps) {
    const navigate = useNavigate();
    const token = getToken();

    const { user, setUser } = UserStore();

    const [visible, setVisible] = useState<boolean>(true);


    const handleSubmit = (e: React.FormEvent<HTMLAnchorElement>) => {
        console.log(username);
        console.log(user.name);

        e.preventDefault();
        fetch(`/api/bans/unban`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                bannedName: username,
                issuerName: user.name
            })

        })
            .then(async (data) => {
                console.log(data);
                if (data.ok) {

                    setVisible(false);
                    if (onUnbanned) onUnbanned();
                }
                else {
                    const body = await data.json()
                    console.log(body);
                    console.error("Unban failed");
                }

            })
            .catch((error) => {
                console.error(error);
            })
    };

    if (!visible) return null;

    return (
        <>
            <a href="#" onClick={handleSubmit}>{children}</a>
        </>
    );
}

export default UnbanLink;
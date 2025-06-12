import React, { useState, useEffect } from 'react';
import { useParams, useSearchParams, useNavigate, Navigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/Navbar.tsx";
import { getToken } from "../Utils/authToken.ts";
import UserStore from '../Stores/UserStore.ts';

function Ban() {
    const param = useParams();
    const [searchParams, setSearchParams] = useSearchParams();
    const [reason, setReason] = useState("");
    const [banned, setBanned] = useState<boolean>(false);
    const [endDate, setEndDate] = useState("");
    const [isPermanent, setIsPermanent] = useState<boolean>(false);
    // state variable for error messages (and also other messages)
    const [errors, setErrors] = useState<string[]>([]);

    const navigate = useNavigate();

    const { user, setUser } = UserStore();
    const username = param.username;
    const reportID = searchParams.get("reportID")

    const token = getToken();

    const setError = (e: string) => setErrors([e]);

    const fetchUser = async () => {
        const response = await fetch(`/api/account/user/${username}`, {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            }
        });


        if (response.ok) {
            let data = await response.json();
            console.log(data);
            setBanned(data.banned);
            setError("");
        }
        else {
            setError("User doesn't exist!");
        }
    };

    useEffect(() => {
        console.log("")
        fetchUser();
    }, [username]);


    const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        const { name, value, type, checked } = e.target
        if (type === "checkbox") {
            setIsPermanent(checked);
            if (checked) setEndDate(""); // Clear end date when permanent
        } else {
            if (name === "reason") setReason(value);
            if (name === "endDate") setEndDate(value);
        }
    };

    const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        if (!reason || (!isPermanent && !endDate)) {
            setError("Please fill in all fields.");
        }
        else if (isPermanent && endDate) {
            setError("Cannot have an end date for a permanent ban!");
        }
        else {
            if (!isPermanent) {
                const selectedDate = new Date(endDate);
                const today = new Date();
                today.setHours(0, 0, 0, 0);

                if (selectedDate <= today) {
                    setError("End date must be in the future.");
                    return;
                }
            }
            setError("");

            fetch("/api/bans/create", {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${token}`,
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    bannedName: username,
                    issuerName: user.name,
                    Reason: reason,
                    permanent: isPermanent,
                    endDate: isPermanent ? null : endDate,
                    reportID : reportID 
                }),
            }).then(async (response) => {
                console.log(response);
                if (response.ok) {
                    setError("Banned successfully");
                }
                else {
                    const data = await response.json();
                    console.log(data);
                    console.log(typeof data.errors);
                    setErrors(data.errors.map((e: { description: string }) => e.description) || ["Error banning."]);
                }
            });
        }
    }

    return (
        <AuthorizeView>
            {
                (user.name === username || banned === true) ?
                    <Navigate to={"/"}></Navigate>
                    :
                    <>
                        <NavBar></NavBar>
                        <div className="containerbox">
                            <h3>Ban menu</h3>
                            <form onSubmit={handleSubmit} name="form">
                                <div>
                                    <label className="forminput" htmlFor="content">Reason:</label>
                                </div>
                                <div>
                                    <textarea id="reason" name="reason" form="form" onInput={handleChange}></textarea>
                                </div>
                                <div>
                                    <input
                                        type="checkbox"
                                        id="permanent"
                                        name="permanent"
                                        checked={isPermanent}
                                        onChange={handleChange}
                                    />
                                    <label htmlFor="permanent">Permanent ban</label>
                                </div>
                                <div>
                                    <label className="forminput" htmlFor="endDate">End Date:</label>
                                </div>
                                <div>
                                    <input
                                        type="date"
                                        id="endDate"
                                        name="endDate"
                                        value={endDate}
                                        onChange={handleChange}
                                        disabled={isPermanent}
                                    />
                                </div>
                                <div>
                                    <button type="submit">Create ban</button>
                                </div>
                            </form>
                            {errors.length > 0 && (
                                <div className="error">
                                    {errors.map((err, index) => (
                                        <p key={index}>{err}</p>
                                    ))}
                                </div>
                            )}
                        </div>
                    </>
            }
        </AuthorizeView >
    );
}


export default Ban;
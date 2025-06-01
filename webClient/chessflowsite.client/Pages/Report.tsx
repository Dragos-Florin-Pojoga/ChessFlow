import React, { useState, useEffect } from 'react';
import { useParams, useSearchParams, useNavigate, Navigate } from "react-router-dom";
import AuthorizeView from "../Components/AuthorizeView.js";
import NavBar from "../Components/NavBar.js";
import { getToken } from "../Utils/authToken.ts";
import UserStore from '../stores/UserStore.ts';

function Report() {
    const param = useParams();
    const [searchParams, setSearchParams] = useSearchParams();
    const [reason, setReason] = useState("");
    // state variable for error messages (and also other messages)
    const [errors, setErrors] = useState<string[]>([]);

    const navigate = useNavigate();

    const { user, setUser } = UserStore();

    console.log(param);
    const username = param.username;
    const gameID = searchParams.get("gameID")

    const token = getToken();

    console.log(username);
    console.log(user.name);

    const setError = (e: string) => setErrors([e]);


    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const { name, value } = e.target;
        if (name === "reason") setReason(value);
    };

    const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        if (!reason) {
            setError("Please fill in all fields.");
        }
        else {
            setError("");

            fetch("/api/reports/create", {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${token}`,
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    ReportedName: username,
                    ReporteeName: user.name,
                    Reason: reason,
                    gameID: gameID || null
                }),
            }).then(async (response) => {
                console.log(response);
                if (response.ok) {
                    setError("Reported successfully");
                    //navigate("/");
                }
                else {
                    const data = await response.json();
                    console.log(data);
                    console.log(typeof data.errors);
                    setErrors(data.errors.map((e: { description: string }) => e.description) || ["Error reporting."]);
                }
            });
        }
    }

    return (
        <AuthorizeView>
            {
                user.name === username ?
                    <Navigate to={"/"}></Navigate>
                    :
                    <>
                        <NavBar></NavBar>
                        <div className="containerbox">
                            <h3>Report menu</h3>
                            <form onSubmit={handleSubmit} name="form">
                                <div>
                                    <label className="forminput" htmlFor="content">Reason:</label>
                                </div>
                                <div>
                                    <textarea id="reason" name="reason" form="form" onInput={handleChange}></textarea>
                                </div>

                                <div>
                                    <button type="submit">Submit report</button>
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


export default Report;
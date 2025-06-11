import React, { useState, useEffect } from "react";
import { useNavigate, Navigate } from "react-router-dom";
import UserStore from '../Stores/UserStore.ts';
import SignalRStore from "../Stores/SignalRStore.ts";
import { getToken, setToken } from "../Utils/authToken.ts";
import Banned from '../Components/Banned.tsx';
import GameStore from '../Stores/GameStore.js';
function Login() {
    const { setUser } = UserStore();
    const { startConnection, stopConnection } = SignalRStore();
    const { clearGame } = GameStore();


    // state variables for email and passwords
    const [email, setEmail] = useState<string>("");
    const [password, setPassword] = useState<string>("");
    const [rememberme, setRememberme] = useState<boolean>(false);
    // state variable for error messages (and also other messages)
    const [errors, setErrors] = useState<string[]>([]);
    const navigate = useNavigate();

    //check if user is already logged in
    const [logged, setLogged] = useState<boolean>(false);
    //check if user is banned
    const [banned, setBanned] = useState<boolean>(false);

    // handle change events for input fields
    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const { name, value } = e.target;
        if (name === "email") setEmail(value);
        if (name === "password") setPassword(value);
        if (name === "rememberme") setRememberme(e.target.checked);
    };

    const handleRegisterClick = () => {
        navigate("/register");
    }



    const setError = (e: string) => setErrors([e]);

    // handle submit event for the form
    const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        // validate email and passwords
        if (!email || !password) {
            setError("Please fill in all fields.");
        } else {
            // clear error message
            setError("");

            fetch("/api/account/login", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    email: email,
                    password: password,
                }),
            })
                .then(async (response) => {
                    console.log(response);
                    if (response.ok) {
                        const data = await response.json();
                        console.log(data);
                        if (data.banned) {
                            if (data.permaban) setError("You're permabanned!");
                            else {
                                let date = new Date(data.bannedUntil+"Z");
                                setError(`You're banned until ${date.toString()}!`);
                            }
                            setBanned(true);
                        }
                        else {
                            const token = data.token;
                            console.log(token);
                            setToken(token, rememberme);

                            fetch("/api/account/pingauth", {
                                method: "GET",
                                headers: {
                                    Authorization: `Bearer ${token}`,
                                    "Content-Type": "application/json",
                                }
                            }).then(async (response) => {
                                if (response.status == 200) {
                                    let j: any = await response.json();
                                    setUser({ email: j.email, name: j.name });
                                }
                                else setError("Store error.")
                            });
                            setError("Login successful.");

                            await stopConnection();  // kill unauthenticated connection
                            await startConnection();

                            navigate("/");
                            setTimeout(() => clearGame(), 100);
                        }
                    }
                    else if (response.status === 401) {
                        const data = await response.json();
                        setErrors([data.message]);
                    }
                    else {
                        const data = await response.json();
                        console.log(data);
                        console.log(typeof data.errors);
                        setErrors(Object.values(data.errors) as string[] || ["Error registering."]);
                    }
                })
                .catch((err) => {
                    console.error(err);
                    setError("Network error.")
                });
        }
    };

    useEffect(() => {
        let token = getToken();
        if (token === null) {
            token = "";
        }
        fetch("/api/account/pingauth", {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            }
        }).then((response) => {
            if (response.status === 200) setLogged(true);
            else setLogged(false);
        });
    }, []);

    if (banned) return (
        <Banned>{errors.length > 0 && (
            <div className="error">
                {errors.map((err, index) => (
                    <p key={index}>{err}</p>
                ))}
            </div>
        )}</Banned>
    );

    return (
        <>
            {
                logged ?
                <Navigate to="/"></Navigate>
                :
            <div className="containerbox">
                <h3>Login</h3>
                <form onSubmit={handleSubmit}>
                    <div>
                        <label className="forminput" htmlFor="email">Email:</label>
                    </div>
                    <div>
                        <input
                            type="email"
                            id="email"
                            name="email"
                            value={email}
                            onChange={handleChange}
                        />
                    </div>
                    <div>
                        <label htmlFor="password">Password:</label>
                    </div>
                    <div>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            value={password}
                            onChange={handleChange}
                        />
                    </div>
                    <div>
                        <input
                            type="checkbox"
                            id="rememberme"
                            name="rememberme"
                            checked={!!rememberme}
                            onChange={handleChange} /><span>Remember Me</span>
                    </div>
                    <div>
                        <button type="submit">Login</button>
                    </div>
                    <div>
                        <button type="button" onClick={handleRegisterClick}>Register</button>
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
            }
        </>
    );
}

export default Login;
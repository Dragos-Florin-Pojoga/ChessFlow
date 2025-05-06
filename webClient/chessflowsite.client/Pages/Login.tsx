import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import UserStore from '../stores/UserStore.ts';
function Login() {
    const { setUser } = UserStore();


    // state variables for email and passwords
    const [email, setEmail] = useState<string>("");
    const [password, setPassword] = useState<string>("");
    const [rememberme, setRememberme] = useState<boolean>(false);
    // state variable for error messages
    const [errors, setErrors] = useState<string[]>([]);
    const navigate = useNavigate();

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
                    console.log(response)
                    if (response.ok) {
                        const data = await response.json();
                        console.log(data);
                        const token = data.token;
                        console.log(token);
                        if (rememberme) {
                            localStorage.setItem("token", token);
                        } else {
                            sessionStorage.setItem("token", token);
                        }
                        setError("Registration successful.");
                        setUser({ email: email });
                        navigate("/");
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

    return (
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
    );
}

export default Login;
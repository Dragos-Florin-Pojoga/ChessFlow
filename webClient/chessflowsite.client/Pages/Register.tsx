import { useState } from "react";
import { useNavigate } from "react-router-dom";


function Register() {
    // state variables for email and passwords
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");
    const [confirmPassword, setConfirmPassword] = useState("");
    const [elo, setElo] = useState(1200);
    const [name1, setName1] = useState("");
    const navigate = useNavigate();

    // state variable for error messages
    const [errors, setErrors] = useState<string[]>([]);

    const handleLoginClick = () => {
        navigate("/login");
    }

    const setError = (e: string) => setErrors([e]);


    // handle change events for input fields
    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const { name, value } = e.target;
        if (name === "email") setEmail(value);
        if (name === "password") setPassword(value);
        if (name === "confirmPassword") setConfirmPassword(value);
        if (name === "name") setName1(value);
    };

    // handle change event for the ELO select dropdown
    const handleEloChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        setElo(Number(e.target.value));
    };

    // handle submit event for the form
    const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        // validate email and passwords
        if (!email || !password || !confirmPassword) {
            setError("Please fill in all fields.");
        } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            setError("Please enter a valid email address.");
        } else if (password !== confirmPassword) {
            setError("Passwords do not match.");
        } else {
            // clear error message
            setError("");
            // post data to the /register api
            fetch("/api/account/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    email: email,
                    password: password,
                    elo: elo,
                    name: name1
                }),
            })
                .then(async (response) => {
                    console.log(response)
                    if (response.ok) {
                        setError("Registration successful.");
                    }
                    else{
                        const data = await response.json();
                        console.log(data);
                        console.log(typeof data.errors);
                        setErrors(data.errors.map((e: any) => e.description) || ["Error registering."]);
                    }
                })
                .catch((err) => {
                    console.error(err);
                    setError(err.message || "Network error.")
                });
        }
    };

    return (
        <div className="containerbox">
            <h3>Register</h3>

            <form onSubmit={handleSubmit}>
                <div>
                    <label htmlFor="email">Email:</label>
                </div><div>
                    <input
                        type="email"
                        id="email"
                        name="email"
                        value={email}
                        onChange={handleChange}
                    />
                </div>
                <div>
                    <label htmlFor="password">Password:</label></div><div>
                    <input
                        type="password"
                        id="password"
                        name="password"
                        value={password}
                        onChange={handleChange}
                    />
                </div>
                <div>
                    <label htmlFor="confirmPassword">Confirm Password:</label></div><div>
                    <input
                        type="password"
                        id="confirmPassword"
                        name="confirmPassword"
                        value={confirmPassword}
                        onChange={handleChange}
                    />
                </div>
                <div>
                    <label htmlFor="name">Username: </label></div><div>
                    <input
                        type="name"
                        id="name"
                        name="name"
                        value={name1}
                        onChange={handleChange}
                    />
                </div>
                <div>
                    <label htmlFor="elo">Starting ELO:</label>
                </div>
                <div>
                    <select id="elo" name="elo" value={elo} onChange={handleEloChange} defaultValue={1200}>
                        <option value={400}>New: 400</option>
                        <option value={800}>Beginner: 800</option>
                        <option value={1200}>Intermediate: 1200</option>
                        <option value={1600}>Advanced: 1600</option>
                        <option value={2000}>Expert: 2000</option>
                    </select>
                </div>
                <div>
                    <button type="submit">Register</button>

                </div>
                <div>
                    <button onClick={handleLoginClick}>Go to Login</button>
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

export default Register;
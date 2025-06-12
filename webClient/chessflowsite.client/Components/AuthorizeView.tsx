import React, { useState, useEffect } from 'react';
import { Navigate } from 'react-router-dom';
import UserStore from '../Stores/UserStore.ts';
import { getToken, clearToken } from "../Utils/authToken.ts";

interface User {
    email: string;
}


function AuthorizeView(props: { children: React.ReactNode }) {

    const [authorized, setAuthorized] = useState<boolean>(false);
    const [loading, setLoading] = useState<boolean>(true); // add a loading state

    //check if user is banned
    const [banned, setBanned] = useState<boolean>(false);

    const { user, setUser, clearUser } = UserStore();


    useEffect(() => {
        // Get the cookie value
        let retryCount = 0; // initialize the retry count
        let maxRetries = 10; // set the maximum number of retries
        let delay: number = 1000; // set the delay in milliseconds

        // define a delay function that returns a promise
        function wait(delay: number) {
            return new Promise((resolve) => setTimeout(resolve, delay));
        }

        const token = getToken();

        if (!token) {
            setAuthorized(false);
            setLoading(false);
            return;
        }

        // define a fetch function that retries until status 200 or 401
        async function fetchWithRetry(url: string, options: any) {
            try {
                // make the fetch request
                let response = await fetch(url, options);

                console.log(response);

                // check the status code
                if (response.status == 200) {
                    let data = await response.json();
                    if (data.banned) {
                        clearToken();
                        fetch("/api/account/logout", {
                            method: "POST",
                            headers: {
                                Authorization: `Bearer ${token}`,
                                "Content-Type": "application/json",
                            },
                            body: ""

                        })
                            .then((data) => {
                                if (data.ok) {
                                    setBanned(true);
                                    clearUser();
                                    return response;
                                }


                            })
                            .catch((error) => {
                                console.error(error);
                            })
                        
                    }
                    console.log("Authorized");
                    setUser({ email: data.email, name: data.name });
                    setAuthorized(true);
                    return response; // return the response
                } else if (response.status == 401) {
                    console.log("Unauthorized");
                    return response; // return the response
                } else {
                    // throw an error to trigger the catch block
                    throw new Error("" + response.status);
                }
            } catch (error) {
                // increment the retry count
                retryCount++;
                // check if the retry limit is reached
                if (retryCount > maxRetries) {
                    // stop retrying and rethrow the error
                    throw error;
                } else {
                    // wait for some time and retry
                    await wait(delay);
                    return fetchWithRetry(url, options);
                }
            }
        }

        // call the fetch function with retry logic
        fetchWithRetry("/api/account/pingauth", {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
                "Content-Type": "application/json",
            }
        })
            .catch((error) => {
                // handle the final error
                console.log(error.message);
            })
            .finally(() => {
                setLoading(false);  // set loading to false when the fetch is done
            });
    }, []);

    if (banned) {
        return (
            <>
                <Navigate to="/" />
            </>
        );   
    }
    else if (loading) {
        return (
            <>
                <p>Loading...</p>
            </>
        );
    }
    else {
        if (authorized && !loading) {
            return (
                 <>{props.children}</>
            );
        } else {
            return (
                <>
                    <Navigate to="/login" />
                </>
            )
        }
    }

}

export function AuthorizedUser(props: { value: string }) {
    // Consume the username from the UserStore
    const { user, setUser } = UserStore();



    // Display the username in a h1 tag
    if (props.value == "email")
        return <>{user.email}</>;
    else if (props.value == "name")
        return <>{user.name}</>;
    else
        return <></>
}

export function UpdateUserInfo(token: string | null) {
    const { user, setUser } = UserStore();

    if (token == null) return false;

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
            return true;
        }
        else return false;
    });
}

export default AuthorizeView;
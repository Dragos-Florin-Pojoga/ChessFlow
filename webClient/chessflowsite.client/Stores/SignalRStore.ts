import { HubConnection, HubConnectionBuilder } from "@microsoft/signalr";
import { create } from "zustand";
import { getToken } from "../Utils/authToken";

interface SignalRStore {
    connection: HubConnection | null;
    startConnection: () => Promise<void>;
    stopConnection: () => Promise<void>;
}

const useSignalRStore = create<SignalRStore>((set) => ({
    connection: null,
    startConnection: async () => {
        const conn = new HubConnectionBuilder()
            .withUrl("/api/gamehub", {
                accessTokenFactory: () => getToken() || "",
            })
            .withAutomaticReconnect()
            .build();

        // retry logic because the client starts up faster than the server in dev, shouldn't be needed in prod (gives a lot of console errors on startup, ugly!)
        const maxRetries = 5;
        let attempt = 0;

        while (attempt < maxRetries) {
            console.log(attempt)
            try {
                await conn.start();
                set({ connection: conn });
                console.log("SignalR connected");
                break; // success, exit loop
            } catch (err) {
                attempt++;
                console.log(`SignalR connection attempt ${attempt} failed:`, err);
                if (attempt >= maxRetries) {
                    console.log("Max SignalR connection attempts reached. Giving up.");
                    break;
                }
                // Wait 1 second before retrying
                await new Promise((res) => setTimeout(res, 1000));
            }
        }
    },
    stopConnection: async () => {
        set((state) => {
            state.connection?.stop();
            return { connection: null };
        });
    }
}));

export default useSignalRStore;
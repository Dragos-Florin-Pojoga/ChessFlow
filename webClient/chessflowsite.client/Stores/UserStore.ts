import { create } from 'zustand';

interface User {
    email: string;
}

interface UserStore {
    user: User;
    setUser: (user: User) => void;
    clearUser: () => void;
}

const useUserStore = create<UserStore>((set) => ({
    user: { email: '' },
    setUser: (user) => set({ user }),
    clearUser: () => set({ user: { email: '' } }),
}));

export default useUserStore;
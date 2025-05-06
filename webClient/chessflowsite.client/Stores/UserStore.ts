import { create } from 'zustand';

interface User {
    email: string;
    name: string;
}

interface UserStore {
    user: User;
    setUser: (user: User) => void;
    clearUser: () => void;
}

const useUserStore = create<UserStore>((set) => ({
    user: { email: '', name: '' },
    setUser: (user) => set({ user }),
    clearUser: () => set({ user: { email: '', name: '' } }),
}));

export default useUserStore;
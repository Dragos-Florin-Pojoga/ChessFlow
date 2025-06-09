import { create } from 'zustand';


type Side = 'w' | 'b';
interface Game {
    id: number;
    side: Side;
    activeSide: Side;
    name: string;
    opponentName: string;
    isBot: boolean;
    elo: number;
    opponentElo: number | null; //null if bot
    isGuest: boolean;
    isOpponentGuest: boolean;
    format: string;
    timer: number;
    opponentTimer: number;
    fen: string;
    isOver: boolean;
}

interface GameStore {
    game: Game | null;
    setGame: (game: Game) => void;
    updateGame: (updates: Partial<Game>) => void;
    setFen: (fen: string) => void;
    clearGame: () => void;
}

const useGameStore = create<GameStore>((set) => ({
    game: null,
    setGame: (game) => set({ game }),
    updateGame: (updates) =>
        set((state) => ({
            game: state.game ? { ...state.game, ...updates } : state.game,
        })),
    setFen: (fen) =>
        set((state) => ({
            game: state.game ? { ...state.game, fen } : state.game,
        })),
    clearGame: () => set({ game: null }),
}));

export default useGameStore;
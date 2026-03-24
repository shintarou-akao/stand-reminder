import { create } from "zustand";

export interface StateSnapshot {
  timerRemainingSecs: number;
}

interface AppStore extends StateSnapshot {
  setFromBackend: (snapshot: Partial<StateSnapshot>) => void;
}

export const useAppStore = create<AppStore>((set) => ({
  timerRemainingSecs: 25 * 60,
  setFromBackend: (snapshot) => set(snapshot),
}));

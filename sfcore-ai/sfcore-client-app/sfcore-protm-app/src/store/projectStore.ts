
import { create } from 'zustand';

interface ProjectStore {
  selectedProjectId: string | null;
  setSelectedProjectId: (id: string | null) => void;
  projects: any[]; // Define proper type later
  fetchProjects: () => Promise<void>;
}

export const useProjectStore = create<ProjectStore>((set: any) => ({
  selectedProjectId: null,
  projects: [],
  setSelectedProjectId: (id: string | null) => set({ selectedProjectId: id }),
  fetchProjects: async () => {
    // TODO: Implement fetch
  },
}));

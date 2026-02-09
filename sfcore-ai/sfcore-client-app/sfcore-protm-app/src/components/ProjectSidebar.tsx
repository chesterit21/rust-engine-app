
import React, { useEffect } from 'react';
import { useProjectStore } from '../store/projectStore';
import { api, endpoints } from '../api/client';

const Sidebar = () => {
  const { selectedProjectId, setSelectedProjectId } = useProjectStore();
  // Using local state for fetch since store implementation is pending full action logic
  const [localProjects, setLocalProjects] = React.useState<any[]>([]);

  useEffect(() => {
    // Fetch projects
    api.get(endpoints.projects)
      .then((res: any) => setLocalProjects(res.data))
      .catch((err: any) => console.error("Failed to fetch projects", err));
  }, []);

  return (
    <aside className="w-64 bg-dark-bg border-r border-dark-border h-full flex flex-col pt-4">
      <div className="px-6 mb-4">
        <h2 className="text-xs font-semibold text-neon-cyan uppercase tracking-widest">
            Projects
        </h2>
      </div>
      <div className="overflow-y-auto flex-1 px-3 space-y-1">
        {localProjects.length === 0 && (
          <div className="text-slate-500 text-sm text-center p-4">
            No projects found.
          </div>
        )}
        {localProjects.map((proj: any) => (
          <button 
            key={proj.id}
            onClick={() => setSelectedProjectId(proj.id)}
            className={`
              w-full text-left px-4 py-3 rounded-xl transition flex items-center gap-3 group
              ${selectedProjectId === proj.id 
                ? 'bg-gradient-to-r from-neon-cyan/20 to-transparent border-l-2 border-neon-cyan text-white shadow-[0_0_15px_rgba(0,240,255,0.1)]' 
                : 'text-slate-400 hover:bg-white/5 hover:text-white'}
            `}
          >
             <div className="font-medium truncate">{proj.name}</div>
          </button>
        ))}
      </div>
    </aside>
  );
};

export default Sidebar;

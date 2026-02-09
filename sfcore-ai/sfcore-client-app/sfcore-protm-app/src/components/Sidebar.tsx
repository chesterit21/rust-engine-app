
import { useEffect, useState } from 'react';
import { useProjectStore } from '../store/projectStore';
import { api, endpoints } from '../api/client';
import { FolderOpen, Box } from 'lucide-react';
import { Link } from 'react-router-dom';

const Sidebar = () => {
  const { selectedProjectId, setSelectedProjectId } = useProjectStore();
  const [localProjects, setLocalProjects] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Fetch projects
    setLoading(true);
    api.get(endpoints.projects)
      .then((res: any) => {
          setLocalProjects(res.data);
          setLoading(false);
      })
      .catch((err: any) => {
          console.error("Failed to fetch projects", err);
          setLoading(false);
      });
  }, []);

  return (
    <div className="w-64 bg-dark-bg border-r border-dark-border h-[calc(100vh-64px)] overflow-y-auto mt-16 fixed left-0 top-0 pt-4 flex flex-col">
       <div className="px-6 mb-4">
         <h2 className="text-xs font-semibold text-neon-cyan uppercase tracking-widest">
            Projects
         </h2>
       </div>
       
       <div className="flex-1 space-y-1 px-3">
          {localProjects.map((proj: any) => (
            <button
              key={proj.id}
              onClick={() => setSelectedProjectId(proj.id)}
              className={`w-full text-left px-4 py-3 rounded-xl transition flex items-center justify-between group ${
                selectedProjectId === proj.id 
                  ? 'bg-gradient-to-r from-neon-cyan/20 to-transparent border-l-2 border-neon-cyan text-white shadow-[0_0_15px_rgba(0,240,255,0.1)]' 
                  : 'text-slate-400 hover:bg-white/5 hover:text-white'
              }`}
            >
              <div className="flex items-center gap-3">
                  <div className={`p-2 rounded-lg ${selectedProjectId === proj.id ? 'text-neon-cyan' : 'text-slate-500 group-hover:text-neon-teal'}`}>
                     <FolderOpen size={18} />
                  </div>
                  <span className="font-medium truncate">{proj.name}</span>
              </div>
              {selectedProjectId === proj.id && (
                  <div className="w-2 h-2 rounded-full bg-neon-cyan shadow-[0_0_5px_#00F0FF]"></div>
              )}
            </button>
          ))}
          
          {localProjects.length === 0 && !loading && (
             <div className="text-center py-8 text-slate-600 px-4 text-sm mt-10">
                <Box size={32} className="mx-auto mb-2 opacity-50" />
                <p>No projects yet.</p>
                <Link to="/projects" className="text-neon-cyan hover:underline mt-2 block">Create One</Link>
             </div>
          )}
       </div>
    </div>
  );
};

export default Sidebar;

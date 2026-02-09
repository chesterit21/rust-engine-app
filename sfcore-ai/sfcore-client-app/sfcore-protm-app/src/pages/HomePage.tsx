
import TreeView from '../components/TreeView';
import { useProjectStore } from '../store/projectStore';
import { useState, useEffect } from 'react'; 
import { Network, Layout, Settings } from 'lucide-react'; 

const HomePage = () => {
  const { selectedProjectId, projects } = useProjectStore(); 
  const [project, setProject] = useState<any>(null); 
  const [viewMode, setViewMode] = useState('tree'); 
  const [isEditModalOpen, setIsEditModalOpen] = useState(false); 

  useEffect(() => {
    if (selectedProjectId && projects) {
      const currentProject = projects.find(p => p.id === selectedProjectId);
      setProject(currentProject);
    } else {
      setProject(null);
    }
  }, [selectedProjectId, projects]);


  return (
    <div className="h-full flex flex-col bg-dark-bg text-slate-200">
       {!selectedProjectId || !project ? ( 
          <div className="flex-1 flex flex-col items-center justify-center p-8 text-center text-slate-500">
             <div className="text-6xl mb-4 opacity-50">📂</div>
             <h2 className="text-xl font-semibold text-slate-400">Select a Project</h2>
             <p className="max-w-md mx-auto mt-2 text-slate-600">
               Select a project from the sidebar to view its architecture tree, 
               or manage your projects via the "Project App" menu.
             </p>
          </div>
       ) : (
         <>
          <div className="px-6 py-3 bg-dark-card border-b border-dark-border shadow-[0_0_15px_rgba(0,240,255,0.1)] flex justify-between items-center z-10 sticky top-0 backdrop-blur-md">
            <div className="flex items-center gap-4">
               <div>
                  <h1 className="text-xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-neon-cyan to-neon-teal">{project.name}</h1>
                  <p className="text-xs text-slate-400 max-w-md truncate">{project.description}</p>
               </div>
            </div>
            
            <div className="flex gap-3">
               <button 
                 onClick={() => setViewMode('tree')}
                 className={`p-2 rounded transition ${viewMode === 'tree' ? 'bg-neon-cyan/20 text-neon-cyan' : 'text-slate-400 hover:bg-white/5 hover:text-white'}`}
                 title="Tree View"
               >
                 <Network size={20} />
               </button>
               <button 
                 onClick={() => setViewMode('kanban')}
                 className={`p-2 rounded transition ${viewMode === 'kanban' ? 'bg-neon-cyan/20 text-neon-cyan' : 'text-slate-400 hover:bg-white/5 hover:text-white'}`}
                 title="Kanban Board"
               >
                 <Layout size={20} />
               </button>
               <div className="h-6 w-px bg-dark-border mx-1"></div>
               <button 
                 onClick={() => setIsEditModalOpen(true)}
                 className="flex items-center gap-2 px-3 py-1.5 text-sm bg-dark-card border border-dark-border hover:border-neon-cyan text-slate-300 hover:text-white rounded transition"
               >
                 <Settings size={16} />
                 <span>Settings</span>
               </button>
            </div>
          </div>

          <div className="flex-1 overflow-hidden relative">
             {viewMode === 'tree' ? (
                <div className="h-full overflow-y-auto p-6">
                   <TreeView />
                </div>
             ) : (
                <div className="h-full flex items-center justify-center text-slate-500">
                   <div className="text-center">
                      <Layout size={48} className="mx-auto mb-4 opacity-20" />
                      <p>Kanban view is coming soon...</p>
                   </div>
                </div>
             )}
          </div>
          
          {isEditModalOpen && (
            <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center backdrop-blur-sm">
                <div className="bg-dark-card p-6 rounded-xl border border-neon-cyan shadow-xl w-96">
                    <h3 className="text-xl text-white mb-4 font-bold">Project Settings</h3>
                    <p className="text-slate-400 mb-6">Settings for {project.name} will be available here.</p>
                    <div className="flex justify-end">
                        <button 
                            onClick={() => setIsEditModalOpen(false)} 
                            className="px-4 py-2 bg-dark-bg border border-dark-border rounded text-slate-300 hover:text-white transition"
                        >
                            Close
                        </button>
                    </div>
                </div>
            </div>
          )}
         </>
       )}
    </div>
  );
};

export default HomePage;

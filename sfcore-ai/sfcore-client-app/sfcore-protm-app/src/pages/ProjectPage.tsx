
import { useEffect, useState } from 'react';
import { api, endpoints } from '../api/client';
import { Plus, Edit2, Trash2, FolderOpen } from 'lucide-react';
import ProjectFormModal from '../components/ProjectFormModal';
import { useProjectStore } from '../store/projectStore';


const ProjectPage = () => {
  const [projects, setProjects] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingProject, setEditingProject] = useState<any | null>(null);
  
  const { setSelectedProjectId } = useProjectStore();


  const fetchProjects = () => {
    setLoading(true);
    api.get(endpoints.projects)
      .then((res: any) => setProjects(res.data))
      .catch((err: any) => console.error(err))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchProjects();
  }, []);

  const handleCreate = async (project: any) => {
    try {
      if (project.id) {
        await api.put(`${endpoints.projects}/${project.id}`, project);
      } else {
        await api.post(endpoints.projects, project);
      }
      fetchProjects();
      setIsModalOpen(false);
    } catch (err) {
      alert("Search failed");
      console.error(err);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Delete this project?")) return;
    try {
      await api.delete(`${endpoints.projects}/${id}`);
      fetchProjects();
    } catch (err) {
      alert("Delete failed");
    }
  };

  const openProject = (id: string) => {
    setSelectedProjectId(id);
    // Maybe show toast or visually indicate selection
    // Or if we want to show TreeView, assume TreeView is loaded below or we navigate somewhere?
    // User requirement: "klik nama project... muncul list-list module bentuk tree-view bro"
    // Since TreeView depends on selectedProjectId, selecting it here should "Activate" it.
    // We can keep TreeView on this page or another. 
    // Let's assume for now we Just Select it. The TreeView component reads the store.
  };

  return (
    <div className="p-8 max-w-6xl mx-auto">
      <div className="flex justify-between items-center mb-8">
        <div>
           <h1 className="text-3xl font-bold text-white bg-gradient-to-r from-neon-cyan to-neon-teal bg-clip-text text-transparent">Projects</h1>
           <p className="text-slate-400 mt-1">Manage your software architecture projects</p>
        </div>
        <button 
          onClick={() => { setEditingProject(null); setIsModalOpen(true); }}
          className="flex items-center gap-2 bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg font-bold px-4 py-2 rounded-lg transition shadow-[0_0_15px_rgba(0,240,255,0.4)] hover:shadow-[0_0_25px_rgba(0,240,255,0.6)]"
        >
          <Plus size={20} />
          <span>New Project</span>
        </button>
      </div>

      {loading && <div className="text-center py-10 text-neon-cyan animate-pulse">Loading projects...</div>}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {projects.map((project) => (
          <div key={project.id} className="bg-dark-card rounded-xl border border-dark-border shadow-lg hover:shadow-[0_0_20px_rgba(0,240,255,0.15)] transition p-6 group relative backdrop-blur-sm">
             <div className="flex justify-between items-start mb-4">
                <div 
                  onClick={() => openProject(project.id)}
                  className="p-3 bg-dark-bg text-neon-cyan rounded-lg cursor-pointer border border-dark-border hover:border-neon-cyan transition shadow-[0_0_10px_rgba(0,240,255,0.1)]"
                >
                  <FolderOpen size={24} />
                </div>
                <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                   <button 
                     onClick={() => { setEditingProject(project); setIsModalOpen(true); }}
                     className="p-2 text-slate-400 hover:text-neon-cyan hover:bg-dark-bg rounded"
                   >
                     <Edit2 size={18} />
                   </button>
                   <button 
                     onClick={() => handleDelete(project.id)}
                     className="p-2 text-slate-400 hover:text-red-500 hover:bg-dark-bg rounded"
                   >
                     <Trash2 size={18} />
                   </button>
                </div>
             </div>
             
             <h3 
               onClick={() => openProject(project.id)}
               className="text-lg font-bold text-slate-200 mb-2 cursor-pointer hover:text-neon-cyan transition-colors"
              >
                {project.name}
             </h3>
             <p className="text-slate-400 text-sm line-clamp-2 h-10">
               {project.description || "No description provided."}
             </p>
             
             <div className="mt-4 pt-4 border-t border-dark-border flex justify-between text-xs text-slate-500">
               <span>Updated recently</span>
               <span>{project.id.substring(0,6)}...</span>
             </div>
          </div>
        ))}
      </div>

      {!loading && projects.length === 0 && (
        <div className="text-center py-20 bg-dark-card/50 rounded-xl border border-dashed border-dark-border">
           <FolderOpen size={48} className="mx-auto text-slate-600 mb-4" />
           <p className="text-slate-400 font-medium">No projects yet</p>
           <button 
             onClick={() => { setEditingProject(null); setIsModalOpen(true); }}
             className="text-neon-cyan text-sm mt-2 hover:underline"
            >
              Create your first project
            </button>
        </div>
      )}

      {isModalOpen && (
        <ProjectFormModal 
          isOpen={isModalOpen}
          initialData={editingProject}
          onClose={() => setIsModalOpen(false)}
          onSubmit={handleCreate}
        />
      )}
    </div>
  );
};

export default ProjectPage;

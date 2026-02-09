
import React, { useEffect, useState } from 'react';
import { useProjectStore } from '../store/projectStore';
import { api, endpoints } from '../api/client';
import { Folders, Edit2, Trash2, FileCode, Plus, Network, Box } from 'lucide-react';

// Simple "Kotak" recursive structure visualizer
const BoxNode = ({ title, children, onDelete, type, id }: { title: string, children?: React.ReactNode, onDelete: (id: string) => void, type: string, id: string }) => {
  return (
    <div className="border border-dark-border bg-dark-card rounded-xl shadow-lg p-6 mb-6 relative hover:shadow-[0_0_20px_rgba(0,240,255,0.1)] transition group">
      
      <div className="flex justify-between items-start mb-4">
        <div className="flex items-center gap-3">
           <div className="p-2 bg-neon-cyan/10 rounded-lg text-neon-cyan">
              {/* Icon based on type, using Folders as a placeholder for now */}
              {type === 'Module' && <Folders size={20} />}
              {type === 'User Story' && <FileCode size={20} />}
              {type === 'Use Case' && <FileCode size={20} />}
           </div>
           <div>
              <h3 className="font-bold text-lg text-white">{title}</h3>
           </div>
        </div>
        <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition">
            <button className="p-1.5 hover:bg-white/10 rounded text-neon-cyan hover:text-white transition">
                <Edit2 size={14} />
            </button>
            <button onClick={() => onDelete(id)} className="p-1.5 hover:bg-white/10 rounded text-red-500 hover:text-red-400 transition">
                <Trash2 size={14} />
            </button>
        </div>
      </div>

      <div className="pl-4 border-l-2 border-dark-border space-y-4">
         {children}
         {!children && (
            <div className="bg-dark-bg/50 border border-dark-border rounded-lg p-4">
               <div className="flex items-center gap-2 mb-2">
                  <FileCode size={16} className="text-neon-teal" />
                  <span className="font-medium text-sm text-slate-200">No {type === 'Module' ? 'stories' : type === 'User Story' ? 'use cases' : 'items'} added yet.</span>
               </div>
               <button className="mt-3 text-xs flex items-center gap-1 text-neon-cyan hover:underline">
                  <Plus size={12} /> Add {type === 'Module' ? 'Story' : type === 'User Story' ? 'Use Case' : 'Item'}
               </button>
            </div>
         )}
      </div>
    </div>
  );
};

const TreeView = () => {
  const { selectedProjectId } = useProjectStore();
  const [data, setData] = useState<{ modules: any[], userStories: any[], useCases: any[] }>({ modules: [], userStories: [], useCases: [] });
  const [loading, setLoading] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);

  useEffect(() => {
    if (!selectedProjectId) return;
    
    setLoading(true);
    
    // Fetch hierarchical data (naive approach: fetch all and filter)
    Promise.all([
      api.get(endpoints.modules),
      api.get(endpoints.userStories),
      api.get(endpoints.useCases)
    ]).then(([modulesRes, storiesRes, useCasesRes]) => {
      setData({
        modules: modulesRes.data.filter((m: any) => m.project_id === selectedProjectId),
        userStories: storiesRes.data,
        useCases: useCasesRes.data
      });
    }).catch(err => {
      console.error("Failed to load hierarchy", err);
    }).finally(() => setLoading(false));

  }, [selectedProjectId]);

  const handleDelete = async (type: string, id: string) => {
    if (!confirm("Are you sure?")) return;
    try {
      let endpoint = '';
      if (type === 'module') endpoint = endpoints.modules;
      else if (type === 'story') endpoint = endpoints.userStories;
      else if (type === 'usecase') endpoint = endpoints.useCases;
      
      await api.delete(`${endpoint}/${id}`);
      
      // Refresh local state (naive)
      setData(prev => {
        if (type === 'module') return { ...prev, modules: prev.modules.filter(m => m.id !== id) };
        if (type === 'story') return { ...prev, userStories: prev.userStories.filter(s => s.id !== id) };
        if (type === 'usecase') return { ...prev, useCases: prev.useCases.filter(u => u.id !== id) };
        return prev;
      });
    } catch (err) {
      alert("Failed to delete");
      console.error(err);
    }
  };

  if (!selectedProjectId) return <div className="p-10 text-center text-slate-400">Select a project to view details</div>;
  if (loading) return <div className="p-10 text-center text-neon-cyan animate-pulse">Loading...</div>;

  return (
    <div className="pb-10">
      <div className="flex justify-between items-center mb-6">
         <h2 className="text-lg font-bold text-white flex items-center gap-2">
            <Network size={20} className="text-neon-cyan"/>
            Module Structure
         </h2>
         <button 
           onClick={() => setIsModalOpen(true)}
           className="bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg font-bold px-3 py-1.5 rounded text-sm flex items-center gap-2 hover:shadow-[0_0_15px_rgba(0,240,255,0.4)] transition"
         >
            <Plus size={16} /> New Module
         </button>
      </div>
      
      {!loading && data.modules.length === 0 && (
         <div className="text-center py-12 border-2 border-dashed border-dark-border rounded-xl bg-dark-bg/50">
            <Box size={40} className="mx-auto mb-3 text-slate-600" />
            <p className="text-slate-400">No modules found. Start building your architecture!</p>
            <button 
               onClick={() => setIsModalOpen(true)}
               className="mt-4 text-neon-cyan hover:underline"
            >
               Create First Module
            </button>
         </div>
      )}

      {data.modules.map((module: any) => (
        <BoxNode 
          key={module.id} 
          id={module.id}
          title={module.name} 
          type="Module"
          onDelete={() => handleDelete('module', module.id)}
        >
          {data.userStories
            .filter((s: any) => s.module_id === module.id)
            .map((story: any) => (
              <BoxNode 
                key={story.id} 
                id={story.id}
                title={story.name} 
                type="User Story"
                onDelete={() => handleDelete('story', story.id)}
              >
                 {data.useCases
                    .filter((u: any) => u.user_story_id === story.id)
                    .map((useCase: any) => (
                      <BoxNode 
                        key={useCase.id} 
                        id={useCase.id}
                        title={useCase.name} 
                        type="Use Case"
                        onDelete={() => handleDelete('usecase', useCase.id)}
                      >
                        {/* Recursive Task/Flow would go here - intentionally empty for now */}
                      </BoxNode>
                    ))}
              </BoxNode>
            ))}
        </BoxNode>
      ))}
      
      {/* Module Form Modal placeholder */}
      {isModalOpen && (
        <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center">
            <div className="bg-dark-card p-6 rounded-xl border border-neon-cyan">
                <h3 className="text-xl text-white mb-4">New Module</h3>
                <p className="text-slate-400 mb-4">Feature coming soon...</p>
                <button onClick={() => setIsModalOpen(false)} className="text-neon-cyan">Close</button>
            </div>
        </div>
      )}
    </div>
  );
};

export default TreeView;

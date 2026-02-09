
import { useState } from 'react';
import { X } from 'lucide-react';

interface Project {
  id?: string;
  name: string;
  description: string;
}

interface ProjectFormModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (project: Project) => void;
  initialData?: Project | null;
}

const ProjectFormModal = ({ isOpen, onClose, onSubmit, initialData }: ProjectFormModalProps) => {
  const [name, setName] = useState(initialData?.name || '');
  const [description, setDescription] = useState(initialData?.description || '');

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit({ id: initialData?.id, name, description });
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center backdrop-blur-md">
      <div className="bg-dark-card rounded-xl border border-neon-cyan/30 shadow-[0_0_50px_rgba(0,0,0,0.5)] w-full max-w-md p-6 relative">
        <button 
          onClick={onClose}
          className="absolute top-4 right-4 text-slate-400 hover:text-white transition"
        >
          <X size={24} />
        </button>
        
        <h2 className="text-xl font-bold mb-4 text-transparent bg-clip-text bg-gradient-to-r from-neon-cyan to-neon-teal">
          {initialData ? 'Edit Project' : 'New Project'}
        </h2>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
           <div>
             <label className="block text-sm font-medium text-slate-300 mb-1">
               Project Name
             </label>
             <input 
               type="text"
               required
               className="w-full border border-dark-border rounded-lg p-2 bg-dark-bg text-white focus:ring-2 focus:ring-neon-cyan focus:border-neon-cyan focus:outline-none placeholder-slate-600"
               value={name}
               onChange={e => setName(e.target.value)}
               placeholder="e.g., E-Commerce Platform"
             />
           </div>

           <div>
             <label className="block text-sm font-medium text-slate-300 mb-1">
               Description
             </label>
             <textarea 
               className="w-full border border-dark-border rounded-lg p-2 bg-dark-bg text-white focus:ring-2 focus:ring-neon-cyan focus:border-neon-cyan focus:outline-none placeholder-slate-600 h-24"
               value={description}
               onChange={e => setDescription(e.target.value)}
               placeholder="Describe the project goals and scope..."
             />
           </div>

           <div className="flex justify-end gap-3 mt-4 pt-4 border-t border-dark-border">
              <button 
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-slate-400 hover:bg-white/5 rounded-lg transition"
              >
                Cancel
              </button>
              <button 
                type="submit"
                className="px-6 py-2 bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg font-bold rounded-lg hover:opacity-90 transition shadow-[0_0_15px_rgba(0,240,255,0.3)]"
              >
                Save Project
              </button>
           </div>
        </form>
      </div>
    </div>
  );
};

export default ProjectFormModal;

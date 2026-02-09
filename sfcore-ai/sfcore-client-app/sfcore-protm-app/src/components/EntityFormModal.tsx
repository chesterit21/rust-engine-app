
import { useState, useEffect } from 'react';
import { X } from 'lucide-react';

interface EntityFormModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: any) => void;
  initialData?: any;
  entityName: string;
}

const EntityFormModal = ({ isOpen, onClose, onSubmit, initialData, entityName }: EntityFormModalProps) => {
  const [formData, setFormData] = useState<any>({});
  
  useEffect(() => {
    if (initialData) {
        setFormData(initialData);
    } else {
        setFormData({});
    }
  }, [initialData]);

  if (!isOpen) return null;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
      setFormData({ ...formData, [e.target.name]: e.target.value });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(formData);
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
          {initialData ? `Edit ${entityName}` : `New ${entityName}`}
        </h2>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
           <div>
             <label className="block text-sm font-medium text-slate-300 mb-1">
               Name
             </label>
             <input 
               type="text"
               name="name"
               required
               className="w-full border border-dark-border rounded-lg p-2 bg-dark-bg text-white focus:ring-2 focus:ring-neon-cyan focus:border-neon-cyan focus:outline-none placeholder-slate-600"
               value={formData.name || ''}
               onChange={handleChange}
               placeholder={`Enter ${entityName} Name`}
             />
           </div>

           <div>
             <label className="block text-sm font-medium text-slate-300 mb-1">
               Description
             </label>
             <textarea 
               name="description"
               className="w-full border border-dark-border rounded-lg p-2 bg-dark-bg text-white focus:ring-2 focus:ring-neon-cyan focus:border-neon-cyan focus:outline-none placeholder-slate-600 h-24"
               value={formData.description || ''}
               onChange={handleChange}
               placeholder="Description..."
             />
           </div>

           {/* Generic fields for other properties could be added here dynamically based on entity schema if available */}

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
                Save
              </button>
           </div>
        </form>
      </div>
    </div>
  );
};

export default EntityFormModal;

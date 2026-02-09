
import { useState, useEffect } from 'react';
import { X, Check, Copy } from 'lucide-react';
import { api } from '../api/client';

interface TechStackFormModalProps {
  isOpen: boolean;
  initialData?: any;
  onClose: () => void;
  onSubmit: (data: any) => void;
}

const TechStackFormModal = ({ isOpen, initialData, onClose, onSubmit }: TechStackFormModalProps) => {
  const [jsonInput, setJsonInput] = useState('');
  const [error, setError] = useState('');
  const [showCopySuccess, setShowCopySuccess] = useState(false);

  useEffect(() => {
    if (initialData) {
      // Normalize stack_type to type for the form (backend DTO expects 'type', but GET returns 'stack_type')
      const { stack_type, ...rest } = initialData;
      const normalizedData = {
          ...rest,
          type: stack_type || rest.type
      };
      setJsonInput(JSON.stringify(normalizedData, null, 2));
    } else {
      setJsonInput('');
    }
    setError('');
    
    // Auto-fetch prompt template if empty and not editing
    if (!initialData && isOpen) {
       api.get('/tech-stacks/prompt').catch(err => console.error("Failed to auto-load prompt", err));
    }
  }, [initialData, isOpen]);

  const validateJson = () => {
    try {
      if (!jsonInput.trim()) {
         setError("Input cannot be empty");
         return false;
      }
      const parsed = JSON.parse(jsonInput);
      
      const validateItem = (item: any) => {
          if (!item.name || !item.type || !item.language) {
              return false;
          }
          return true;
      };

      if (Array.isArray(parsed)) {
          if (parsed.length === 0) {
              setError("Array cannot be empty");
              return false;
          }
          for (let i = 0; i < parsed.length; i++) {
              if (!validateItem(parsed[i])) {
                  setError(`Item at index ${i} is missing required fields (name, type, language)`);
                  return false;
              }
          }
      } else {
          if (!validateItem(parsed)) {
              setError("Missing required fields: name, type, language");
              return false;
          }
      }

      setError('');
      return true;
    } catch (e: any) {
      setError("Invalid JSON format: " + e.message);
      return false;
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validateJson()) {
      const payload = JSON.parse(jsonInput);
      // If editing, preserve ID
      if (initialData?.id) {
         payload.id = initialData.id;
      }
      onSubmit(payload);
      onClose();
    }
  };
  
  const handleLoadPrompt = async () => {
      try {
          const res = await api.get('/tech-stacks/prompt');
          setJsonInput(res.data); // Expecting raw string prompt from backend
      } catch (err) {
          console.error(err);
      }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(jsonInput).then(() => {
      setShowCopySuccess(true);
      setTimeout(() => setShowCopySuccess(false), 2000);
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/80 z-50 flex items-center justify-center backdrop-blur-sm p-4">
      <div className="bg-dark-card rounded-xl border border-neon-cyan shadow-[0_0_30px_rgba(0,240,255,0.15)] w-full max-w-5xl flex flex-col max-h-[90vh]">
        <div className="flex justify-between items-center p-6 border-b border-dark-border">
          <h2 className="text-xl font-bold text-white">
            {initialData ? 'Edit Tech Stack' : 'New Tech Stack'}
          </h2>
          <button onClick={onClose} className="text-slate-400 hover:text-white transition">
            <X size={24} />
          </button>
        </div>

        <div className="p-6 flex-1 overflow-y-auto">
           <div className="mb-4">
             <div className="flex justify-between items-center mb-2">
                <label className="block text-sm font-medium text-slate-300">
                    JSON Configuration / Prompt
                </label>
                <div className="flex gap-2">
                    <button 
                        type="button" 
                        onClick={handleLoadPrompt}
                        className="text-xs text-neon-cyan hover:underline"
                    >
                        Load AI Prompt
                    </button>
                    <button 
                        type="button" 
                        onClick={() => {
                            setJsonInput(JSON.stringify({
                                name: "Example Stack",
                                type: "FULLSTACK",
                                language: "TypeScript",
                                description: "Description here"
                            }, null, 2));
                        }}
                        className="text-xs text-neon-cyan hover:underline"
                    >
                        Load Template
                    </button>
                </div>
             </div>
             <textarea
               className="w-full h-96 bg-dark-bg border border-dark-border rounded-lg p-4 font-mono text-sm text-slate-200 focus:ring-2 focus:ring-neon-cyan focus:border-transparent outline-none resize-none"
               value={jsonInput}
               onChange={(e) => {
                   setJsonInput(e.target.value);
               }}
               placeholder="// Paste JSON here or load prompt..."
             />
             {error && <p className="mt-2 text-sm text-red-400">{error}</p>}
           </div>
        </div>

        <div className="p-6 border-t border-dark-border bg-dark-card rounded-b-xl flex justify-between items-center">
          <div className="flex gap-3">
             <button
                type="button"
                onClick={validateJson}
                className="px-4 py-2 bg-dark-bg border border-dark-border text-slate-200 rounded hover:bg-white/5 transition flex items-center gap-2"
             >
                <Check size={16} /> Validate Answer
             </button>
             <button
                type="button"
                onClick={handleCopy}
                className="px-4 py-2 bg-dark-bg border border-dark-border text-slate-200 rounded hover:bg-white/5 transition flex items-center gap-2 relative"
             >
                <Copy size={16} /> 
                {showCopySuccess ? <span className="text-neon-cyan">Copied!</span> : <span>Copy</span>}
             </button>
          </div>
          
          <div className="flex gap-3">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-slate-400 hover:text-white transition"
              >
                Cancel
              </button>
              <button
                onClick={handleSubmit}
                className={`px-6 py-2 rounded font-bold shadow-lg transition
                  ${jsonInput.trim() 
                    ? 'bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg hover:shadow-neon-cyan/50' 
                    : 'bg-dark-border text-slate-500 cursor-not-allowed'
                  }
                `}
                disabled={!jsonInput.trim()}
              >
                Save
              </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default TechStackFormModal;

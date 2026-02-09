
import React, { useEffect } from 'react';
import { X } from 'lucide-react';

interface RightDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

const RightDrawer: React.FC<RightDrawerProps> = ({ isOpen, onClose, title, children }) => {
  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', handleEsc);
    return () => window.removeEventListener('keydown', handleEsc);
  }, [onClose]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex justify-end">
        {/* Backdrop */}
        <div 
            className="absolute inset-0 bg-black/50 backdrop-blur-sm transition-opacity" 
            onClick={onClose}
        />

        {/* Drawer Content */}
        <div className="relative w-1/3 h-full bg-[#0b0f19] border-l border-white/5 shadow-2xl transform transition-transform duration-300 overflow-y-auto flex flex-col">
             <div className="p-6 border-b border-white/5 flex justify-between items-center bg-[#0b0f19]/90 backdrop-blur sticky top-0 z-10">
                <h2 className="text-xl font-bold text-white bg-gradient-to-r from-neon-cyan to-neon-teal bg-clip-text text-transparent">
                    {title}
                </h2>
                <button 
                    onClick={onClose}
                    className="text-slate-400 hover:text-white transition"
                >
                    <X size={24} />
                </button>
             </div>
             <div className="p-6 flex-1">
                 {children}
             </div>
        </div>
    </div>
  );
};

export default RightDrawer;

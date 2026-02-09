
import { NavLink } from 'react-router-dom';
import { entities } from '../utils/entityConfig';

const AdminSidebar = () => {
  return (
    <aside className="w-64 border-r border-white/5 h-full flex flex-col text-slate-300 bg-[#02040a]/70 backdrop-blur-xl shadow-[1px_0_20px_rgba(255,255,255,0.05)]">
      <div className="p-4 border-b border-white/5 font-bold text-neon-cyan uppercase tracking-wider text-xs">
        Master Data
      </div>
      <div className="overflow-y-auto flex-1 p-2 space-y-2">
        {entities
          .filter(entity => ![
            'attributes', 
            'entities', 
            'entity-relationships', 
            'flow-steps', 
            'tasks', 
            'task-dependencies', 
            'task-entity-usage', 
            'task-file-mappings', 
            'use-cases', 
            'user-stories'
          ].includes(entity.slug))
          .map((entity) => (
          <NavLink
            key={entity.slug}
            to={`/${entity.slug}`}
            className={({ isActive }) => `
              relative group block rounded overflow-hidden transition-all duration-300
              ${isActive ? 'shadow-[0_0_20px_rgba(0,240,255,0.2)]' : ''}
            `}
          >
            {({ isActive }) => (
                <>
                    {/* Layer 1: Gradient Border - Crimson, Purple, Yellow, Green */}
                    <div className={`absolute inset-0 bg-gradient-to-r from-[#DC143C] via-purple-500 via-yellow-400 to-green-500 transition-opacity duration-300 
                        ${isActive ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'}`} 
                    />
                    
                    {/* Layer 2: Solid Background Mask (Matches Global Dark BG #02040a) */}
                    <div className="absolute inset-px rounded bg-[#02040a] transition-all duration-300" />

                    {/* Layer 3: Content */}
                    <div className={`
                        relative rounded px-3 py-2 text-sm font-medium transition-colors
                        flex items-center z-10
                        ${isActive 
                            ? 'text-white' 
                            : 'text-slate-400 group-hover:text-white'}
                    `}>
                        {entity.name}
                    </div>
                </>
            )}
          </NavLink>
        ))}
      </div>
    </aside>
  );
};

export default AdminSidebar;

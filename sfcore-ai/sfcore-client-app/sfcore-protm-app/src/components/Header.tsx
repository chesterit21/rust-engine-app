
import { Link, NavLink } from 'react-router-dom';
import { Grid } from 'lucide-react';

const Header = () => {
  return (
    <header className="fixed top-0 left-0 right-0 h-16 bg-[#02040a]/70 backdrop-blur-xl border-b border-white/5 z-50 flex items-center justify-between px-6 shadow-[0_4px_30px_rgba(0,0,0,0.5)]">
      <div className="flex items-center gap-2">
        <Link to="/" className="text-2xl font-bold bg-gradient-to-r from-[#DC143C] via-red-500 to-orange-500 bg-clip-text text-transparent hover:opacity-80 transition drop-shadow-[0_0_10px_rgba(220,20,60,0.5)]">
          SFCore ProTM
        </Link>
      </div>

      {/* Direct Link to Project Setup */}
      <NavLink 
        to="/projects"
        className={({ isActive }) => `flex items-center gap-2 px-4 py-2 rounded-lg transition border ${
          isActive 
            ? 'bg-neon-cyan/10 border-neon-cyan text-neon-cyan shadow-[0_0_10px_rgba(0,240,255,0.3)]' 
            : 'border-transparent text-slate-300 hover:text-white hover:bg-white/5'
        }`}
      >
        <Grid size={20} />
        <span className="font-medium">Project App Setup</span>
      </NavLink>
    </header>
  );
};

export default Header;

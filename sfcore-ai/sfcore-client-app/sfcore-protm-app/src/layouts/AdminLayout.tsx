
import { Outlet } from 'react-router-dom';
import Header from '../components/Header';
import AdminSidebar from '../components/AdminSidebar';

const AdminLayout = () => {
  return (
    <div className="flex flex-col h-screen overflow-hidden font-sans text-slate-200 bg-dark-bg relative selection:bg-neon-cyan/30 selection:text-neon-cyan">
      {/* Ambient Background Glow - Lime & Yellow */}
      <div className="absolute top-[-20%] left-[-10%] w-[500px] h-[500px] bg-lime-500/15 rounded-full blur-[120px] pointer-events-none" />
      <div className="absolute bottom-[-20%] right-[-10%] w-[500px] h-[500px] bg-yellow-500/15 rounded-full blur-[120px] pointer-events-none" />
      
      <Header />
      <div className="flex flex-1 overflow-hidden pt-16 z-10">
        <AdminSidebar />
        <main className="flex-1 overflow-y-auto relative bg-transparent">
          <Outlet />
        </main>
      </div>
    </div>
  );
};

export default AdminLayout;

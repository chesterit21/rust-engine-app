
import { Outlet } from 'react-router-dom';
import Header from '../components/Header';
import ProjectSidebar from '../components/ProjectSidebar';

const HomeLayout = () => {
  return (
    <div className="flex flex-col h-screen overflow-hidden font-sans text-slate-200 bg-dark-bg">
      <Header />
      <div className="flex flex-1 overflow-hidden pt-16">
        <ProjectSidebar />
        <main className="flex-1 overflow-y-auto relative bg-dark-bg">
          <Outlet />
        </main>
      </div>
    </div>
  );
};

export default HomeLayout;

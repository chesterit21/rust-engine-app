
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import HomeLayout from './layouts/HomeLayout';
import AdminLayout from './layouts/AdminLayout';
import HomePage from './pages/HomePage'; // This import is no longer used in the new structure
import EntityListPage from './pages/EntityListPage';
import ProjectPage from './pages/ProjectPage';
import TechStackPage from './pages/TechStackPage';
import MasterPersonasPage from './pages/MasterPersonasPage';
import ArchitecturePatternsPage from './pages/ArchitecturePatternsPage';

const App = () => {
    return (
        <BrowserRouter> {/* Changed from <Router> to <BrowserRouter> to match original import */}
            <Routes>
                <Route path="/" element={<AdminLayout />}>
                    <Route index element={<Navigate to="/projects" replace />} />
                    <Route path="projects" element={<ProjectPage />} />
                    <Route path="tech-stacks" element={<TechStackPage />} />
                    <Route path="personas" element={<MasterPersonasPage />} />
                    <Route path="architecture-patterns" element={<ArchitecturePatternsPage />} />
                    
                    {/* Generic Route for other entities */}
                    <Route path=":slug" element={<EntityListPage />} />
                </Route>
            </Routes>
        </BrowserRouter>
    );
};

export default App;

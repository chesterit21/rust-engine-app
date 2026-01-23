import { Routes, Route, Navigate } from 'react-router-dom';
import { DashboardLayout } from './components/layout/DashboardLayout';
import { Landing } from './pages/Landing';
import { ProtectedRoute } from './contexts/AuthContext';
import { Dashboard } from './pages/Dashboard';
import { UserList } from './pages/users/UserList';
import { GroupList } from './pages/groups/GroupList';
import { MenuList } from './pages/menus/MenuList';
import { TenantList } from './pages/tenants/TenantList';
import { ClientAppList } from './pages/client-apps/ClientAppList';

function App() {
  return (
    <Routes>
      {/* Public Landing Page */}
      <Route path="/" element={<Landing />} />

      {/* Protected Dashboard Routes */}
      <Route path="/dashboard" element={
        <ProtectedRoute>
          <DashboardLayout />
        </ProtectedRoute>
      }>
        <Route index element={<Dashboard />} />
        <Route path="users" element={<UserList />} />
        <Route path="groups" element={<GroupList />} />
        <Route path="menus" element={<MenuList />} />
        <Route path="tenants" element={<TenantList />} />
        <Route path="client-apps" element={<ClientAppList />} />
      </Route>

      {/* Fallback */}
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export default App;

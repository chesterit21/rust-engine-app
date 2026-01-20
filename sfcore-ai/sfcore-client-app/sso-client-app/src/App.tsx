import { Routes, Route, Navigate } from 'react-router-dom';
import { DashboardLayout } from './components/layout/DashboardLayout';
import { Dashboard } from './pages/Dashboard';
import { UserList } from './pages/users/UserList';
import { GroupList } from './pages/groups/GroupList';
import { MenuList } from './pages/menus/MenuList';

function App() {
  return (
    <Routes>
      {/* Dashboard Routes */}
      <Route path="/" element={<DashboardLayout />}>
        <Route index element={<Dashboard />} />
        <Route path="users" element={<UserList />} />
        <Route path="groups" element={<GroupList />} />
        <Route path="menus" element={<MenuList />} />
      </Route>

      {/* Fallback */}
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export default App;

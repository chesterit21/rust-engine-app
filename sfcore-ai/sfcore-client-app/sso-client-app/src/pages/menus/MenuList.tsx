import React, { useState } from 'react';
import { Plus, Search, ChevronRight, ChevronDown, Edit, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';

// Mock data - tree structure
const mockMenus = [
  {
    id: '1',
    name: 'Dashboard',
    url: '/dashboard',
    icon: 'LayoutDashboard',
    level: 0,
    order: 1,
    is_active: true,
    children: [],
  },
  {
    id: '2',
    name: 'User Management',
    url: '/users',
    icon: 'Users',
    level: 0,
    order: 2,
    is_active: true,
    children: [
      { id: '2-1', name: 'User List', url: '/users', icon: 'Users', level: 1, order: 1, is_active: true, children: [] },
      { id: '2-2', name: 'User Groups', url: '/users/groups', icon: 'UserCog', level: 1, order: 2, is_active: true, children: [] },
    ],
  },
  {
    id: '3',
    name: 'Settings',
    url: '/settings',
    icon: 'Settings',
    level: 0,
    order: 3,
    is_active: true,
    children: [
      { id: '3-1', name: 'General', url: '/settings/general', icon: 'Settings', level: 1, order: 1, is_active: true, children: [] },
      { id: '3-2', name: 'Security', url: '/settings/security', icon: 'Shield', level: 1, order: 2, is_active: true, children: [] },
    ],
  },
];

interface MenuItemProps {
  menu: typeof mockMenus[0];
  depth?: number;
}

const MenuItem: React.FC<MenuItemProps> = ({ menu, depth = 0 }) => {
  const [isExpanded, setIsExpanded] = useState(true);
  const hasChildren = menu.children && menu.children.length > 0;

  return (
    <div>
      <div
        className={`
          flex items-center gap-2 px-4 py-3 
          hover:bg-gray-50 dark:hover:bg-dark-lighter 
          border-b border-gray-100 dark:border-dark-lighter
          transition-colors
        `}
        style={{ paddingLeft: `${depth * 24 + 16}px` }}
      >
        {hasChildren ? (
          <button onClick={() => setIsExpanded(!isExpanded)} className="p-1">
            {isExpanded ? (
              <ChevronDown className="w-4 h-4 text-gray-500" />
            ) : (
              <ChevronRight className="w-4 h-4 text-gray-500" />
            )}
          </button>
        ) : (
          <div className="w-6" />
        )}
        
        <div className="flex-1">
          <span className="font-medium text-gray-900 dark:text-white">{menu.name}</span>
          <span className="ml-2 text-sm text-gray-500 dark:text-gray-400">{menu.url}</span>
        </div>
        
        <span
          className={`px-2 py-0.5 rounded text-xs font-medium ${
            menu.is_active
              ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
              : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
          }`}
        >
          {menu.is_active ? 'Active' : 'Inactive'}
        </span>
        
        <div className="flex gap-1">
          <button className="p-1.5 hover:bg-gray-100 dark:hover:bg-dark-lighter rounded transition-colors">
            <Edit className="w-4 h-4 text-gray-500" />
          </button>
          <button className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors">
            <Trash2 className="w-4 h-4 text-red-500" />
          </button>
        </div>
      </div>
      
      {isExpanded && hasChildren && (
        <div>
          {menu.children.map((child) => (
            <MenuItem key={child.id} menu={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
};

export const MenuList: React.FC = () => {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">
            Menu Management
          </h1>
          <p className="mt-1 text-gray-500 dark:text-gray-400">
            Manage navigation menu structure
          </p>
        </div>
        <Button onClick={() => setIsModalOpen(true)}>
          <Plus className="w-4 h-4" />
          Add Menu
        </Button>
      </div>

      {/* Search */}
      <div className="max-w-md">
        <Input
          placeholder="Search menus..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          icon={<Search className="w-5 h-5" />}
        />
      </div>

      {/* Menu Tree */}
      <div className="bg-white dark:bg-dark-light rounded-xl shadow-sm border border-gray-200 dark:border-dark-lighter overflow-hidden">
        <div className="px-4 py-3 border-b border-gray-200 dark:border-dark-lighter bg-gray-50 dark:bg-dark-lighter">
          <div className="flex items-center gap-4 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
            <span className="flex-1">Menu Item</span>
            <span className="w-20">Status</span>
            <span className="w-20">Actions</span>
          </div>
        </div>
        
        <div>
          {mockMenus.map((menu) => (
            <MenuItem key={menu.id} menu={menu} />
          ))}
        </div>
      </div>

      {/* Create Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title="Create Menu"
        size="md"
      >
        <form className="space-y-4">
          <Input label="Menu Name" placeholder="Enter menu name" />
          <Input label="URL" placeholder="/path/to/page" />
          <Input label="Icon" placeholder="e.g., Users, Settings" />
          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setIsModalOpen(false)}>
              Cancel
            </Button>
            <Button type="submit">Create Menu</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
};

// ============================================================================
// Menu List Page with Tree View
// File: src/pages/menus/MenuList.tsx
// Features: Hierarchical tree, Expand/Collapse, CRUD Modal, Drag-drop ready
// ============================================================================

import React, { useState } from 'react';
import { 
  Plus, Edit, Trash2, ChevronRight, ChevronDown, 
  GripVertical, ExternalLink, Eye, EyeOff
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';

// ============================================================================
// Types
// ============================================================================

interface MenuItem {
  id: string;
  menu_name: string;
  menu_url: string;
  menu_icon: string;
  menu_order: number;
  level: number;
  parent_id: string | null;
  is_active: boolean;
  created_at: string;
  children?: MenuItem[];
}

// ============================================================================
// Mock Data (Hierarchical)
// ============================================================================

const generateMockMenus = (): MenuItem[] => {
  return [
    {
      id: 'menu-001',
      menu_name: 'Dashboard',
      menu_url: '/dashboard',
      menu_icon: 'fa-home',
      menu_order: 1,
      level: 0,
      parent_id: null,
      is_active: true,
      created_at: new Date().toISOString(),
      children: []
    },
    {
      id: 'menu-002',
      menu_name: 'User Management',
      menu_url: '/users',
      menu_icon: 'fa-users',
      menu_order: 2,
      level: 0,
      parent_id: null,
      is_active: true,
      created_at: new Date().toISOString(),
      children: [
        {
          id: 'menu-002-1',
          menu_name: 'User List',
          menu_url: '/users',
          menu_icon: 'fa-list',
          menu_order: 1,
          level: 1,
          parent_id: 'menu-002',
          is_active: true,
          created_at: new Date().toISOString(),
          children: []
        },
        {
          id: 'menu-002-2',
          menu_name: 'User Groups',
          menu_url: '/users/groups',
          menu_icon: 'fa-users-gear',
          menu_order: 2,
          level: 1,
          parent_id: 'menu-002',
          is_active: true,
          created_at: new Date().toISOString(),
          children: []
        },
        {
          id: 'menu-002-3',
          menu_name: 'Permissions',
          menu_url: '/users/permissions',
          menu_icon: 'fa-shield-halved',
          menu_order: 3,
          level: 1,
          parent_id: 'menu-002',
          is_active: false,
          created_at: new Date().toISOString(),
          children: []
        }
      ]
    },
    {
      id: 'menu-003',
      menu_name: 'Settings',
      menu_url: '/settings',
      menu_icon: 'fa-gear',
      menu_order: 3,
      level: 0,
      parent_id: null,
      is_active: true,
      created_at: new Date().toISOString(),
      children: [
        {
          id: 'menu-003-1',
          menu_name: 'General',
          menu_url: '/settings/general',
          menu_icon: 'fa-sliders',
          menu_order: 1,
          level: 1,
          parent_id: 'menu-003',
          is_active: true,
          created_at: new Date().toISOString(),
          children: []
        },
        {
          id: 'menu-003-2',
          menu_name: 'Security',
          menu_url: '/settings/security',
          menu_icon: 'fa-lock',
          menu_order: 2,
          level: 1,
          parent_id: 'menu-003',
          is_active: true,
          created_at: new Date().toISOString(),
          children: [
            {
              id: 'menu-003-2-1',
              menu_name: 'Two-Factor Auth',
              menu_url: '/settings/security/2fa',
              menu_icon: 'fa-mobile',
              menu_order: 1,
              level: 2,
              parent_id: 'menu-003-2',
              is_active: true,
              created_at: new Date().toISOString(),
              children: []
            },
            {
              id: 'menu-003-2-2',
              menu_name: 'API Keys',
              menu_url: '/settings/security/api-keys',
              menu_icon: 'fa-key',
              menu_order: 2,
              level: 2,
              parent_id: 'menu-003-2',
              is_active: true,
              created_at: new Date().toISOString(),
              children: []
            }
          ]
        }
      ]
    },
    {
      id: 'menu-004',
      menu_name: 'Reports',
      menu_url: '/reports',
      menu_icon: 'fa-chart-line',
      menu_order: 4,
      level: 0,
      parent_id: null,
      is_active: true,
      created_at: new Date().toISOString(),
      children: []
    }
  ];
};

// ============================================================================
// Tree Item Component
// ============================================================================

interface TreeItemProps {
  item: MenuItem;
  depth: number;
  expandedIds: Set<string>;
  onToggle: (id: string) => void;
  onEdit: (item: MenuItem) => void;
  onDelete: (item: MenuItem) => void;
}

const TreeItem: React.FC<TreeItemProps> = ({ 
  item, depth, expandedIds, onToggle, onEdit, onDelete 
}) => {
  const hasChildren = item.children && item.children.length > 0;
  const isExpanded = expandedIds.has(item.id);

  return (
    <>
      <div
        className={`
          flex items-center gap-2 px-4 py-3 
          hover:bg-gray-50 dark:hover:bg-dark-lighter 
          border-b border-gray-100 dark:border-dark-lighter
          transition-colors group
        `}
        style={{ paddingLeft: `${depth * 24 + 16}px` }}
      >
        {/* Drag Handle */}
        <div className="opacity-0 group-hover:opacity-100 transition-opacity cursor-grab">
          <GripVertical className="w-4 h-4 text-gray-400" />
        </div>

        {/* Expand/Collapse */}
        {hasChildren ? (
          <button 
            onClick={() => onToggle(item.id)} 
            className="p-1 hover:bg-gray-200 dark:hover:bg-dark-light rounded transition-colors"
          >
            {isExpanded ? (
              <ChevronDown className="w-4 h-4 text-gray-500" />
            ) : (
              <ChevronRight className="w-4 h-4 text-gray-500" />
            )}
          </button>
        ) : (
          <div className="w-6" />
        )}

        {/* Icon */}
        <div className={`
          w-8 h-8 rounded-lg flex items-center justify-center text-sm
          ${item.is_active 
            ? 'bg-primary-100 text-primary-600 dark:bg-primary-900/30 dark:text-primary-400' 
            : 'bg-gray-100 text-gray-400 dark:bg-dark-lighter'}
        `}>
          <i className={`fas ${item.menu_icon}`}></i>
        </div>

        {/* Menu Name & URL */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className={`font-medium ${item.is_active ? 'text-gray-900 dark:text-white' : 'text-gray-400'}`}>
              {item.menu_name}
            </span>
            {!item.is_active && (
              <span className="px-1.5 py-0.5 text-xs bg-gray-100 text-gray-500 dark:bg-dark-lighter rounded">
                Hidden
              </span>
            )}
          </div>
          <div className="flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400">
            <ExternalLink className="w-3 h-3" />
            {item.menu_url}
          </div>
        </div>

        {/* Level Badge */}
        <span className="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-dark-lighter dark:text-gray-400 rounded">
          L{item.level}
        </span>

        {/* Order */}
        <span className="text-sm text-gray-500 w-8 text-center">
          #{item.menu_order}
        </span>

        {/* Actions */}
        <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={() => onEdit(item)}
            className="p-1.5 hover:bg-gray-200 dark:hover:bg-dark-light rounded-lg transition-colors"
            title="Edit"
          >
            <Edit className="w-4 h-4 text-gray-500" />
          </button>
          <button
            onClick={() => onDelete(item)}
            className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
            title="Delete"
          >
            <Trash2 className="w-4 h-4 text-red-500" />
          </button>
        </div>
      </div>

      {/* Children */}
      {hasChildren && isExpanded && (
        <div className="animate-fade-in">
          {item.children!.map(child => (
            <TreeItem
              key={child.id}
              item={child}
              depth={depth + 1}
              expandedIds={expandedIds}
              onToggle={onToggle}
              onEdit={onEdit}
              onDelete={onDelete}
            />
          ))}
        </div>
      )}
    </>
  );
};

// ============================================================================
// Menu Form Component
// ============================================================================

interface MenuFormProps {
  menu?: MenuItem | null;
  parentOptions: { id: string; name: string; level: number }[];
  onSubmit: (data: Partial<MenuItem>) => void;
  onCancel: () => void;
}

const MenuForm: React.FC<MenuFormProps> = ({ menu, parentOptions, onSubmit, onCancel }) => {
  const [formData, setFormData] = useState({
    menu_name: menu?.menu_name || '',
    menu_url: menu?.menu_url || '',
    menu_icon: menu?.menu_icon || 'fa-circle',
    menu_order: menu?.menu_order || 1,
    parent_id: menu?.parent_id || '',
    is_active: menu?.is_active ?? true,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = () => {
    const newErrors: Record<string, string> = {};
    if (!formData.menu_name.trim()) newErrors.menu_name = 'Menu name is required';
    if (!formData.menu_url.trim()) newErrors.menu_url = 'URL is required';
    if (!formData.menu_url.startsWith('/')) newErrors.menu_url = 'URL must start with /';
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validate()) {
      onSubmit({
        ...formData,
        parent_id: formData.parent_id || null,
        level: formData.parent_id ? (parentOptions.find(p => p.id === formData.parent_id)?.level ?? 0) + 1 : 0,
      });
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Input
        label="Menu Name"
        placeholder="e.g., Dashboard"
        value={formData.menu_name}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, menu_name: e.target.value })}
        error={errors.menu_name}
      />

      <Input
        label="URL"
        placeholder="/path/to/page"
        value={formData.menu_url}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, menu_url: e.target.value })}
        error={errors.menu_url}
        icon={<ExternalLink className="w-4 h-4" />}
      />

      <Input
        label="Icon (Font Awesome class)"
        placeholder="fa-home"
        value={formData.menu_icon}
        onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, menu_icon: e.target.value })}
      />

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-1">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Parent Menu
          </label>
          <select
            value={formData.parent_id}
            onChange={(e) => setFormData({ ...formData, parent_id: e.target.value })}
            className="w-full px-4 py-2.5 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 dark:bg-dark-lighter dark:border-dark-lighter dark:text-white"
          >
            <option value="">— Root Level —</option>
            {parentOptions.map(opt => (
              <option key={opt.id} value={opt.id}>
                {'—'.repeat(opt.level)} {opt.name}
              </option>
            ))}
          </select>
        </div>

        <Input
          label="Order"
          type="number"
          value={formData.menu_order.toString()}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, menu_order: parseInt(e.target.value) || 1 })}
        />
      </div>

      <div className="flex items-center gap-3">
        <input
          type="checkbox"
          id="is_active"
          checked={formData.is_active}
          onChange={(e) => setFormData({ ...formData, is_active: e.target.checked })}
          className="w-4 h-4 text-primary-600 rounded border-gray-300 focus:ring-primary-500"
        />
        <label htmlFor="is_active" className="text-sm text-gray-700 dark:text-gray-300 flex items-center gap-2">
          {formData.is_active ? <Eye className="w-4 h-4" /> : <EyeOff className="w-4 h-4" />}
          Menu is visible
        </label>
      </div>

      <div className="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-dark-lighter">
        <Button type="button" variant="secondary" onClick={onCancel}>Cancel</Button>
        <Button type="submit">{menu ? 'Update Menu' : 'Create Menu'}</Button>
      </div>
    </form>
  );
};

// ============================================================================
// Main MenuList Component
// ============================================================================

export const MenuList: React.FC = () => {
  const [menus, _setMenus] = useState<MenuItem[]>(generateMockMenus);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set(['menu-002', 'menu-003']));
  const [isFormModalOpen, setIsFormModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const [selectedMenu, setSelectedMenu] = useState<MenuItem | null>(null);

  // Flatten menus for parent options
  const flattenMenus = (items: MenuItem[], result: { id: string; name: string; level: number }[] = []): { id: string; name: string; level: number }[] => {
    items.forEach(item => {
      result.push({ id: item.id, name: item.menu_name, level: item.level });
      if (item.children) flattenMenus(item.children, result);
    });
    return result;
  };

  // Count total menus
  const countMenus = (items: MenuItem[]): number => {
    return items.reduce((acc, item) => acc + 1 + (item.children ? countMenus(item.children) : 0), 0);
  };

  // Handlers
  const handleToggle = (id: string) => {
    setExpandedIds(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleExpandAll = () => {
    const allIds = new Set<string>();
    const collectIds = (items: MenuItem[]) => {
      items.forEach(item => {
        if (item.children && item.children.length > 0) {
          allIds.add(item.id);
          collectIds(item.children);
        }
      });
    };
    collectIds(menus);
    setExpandedIds(allIds);
  };

  const handleCollapseAll = () => setExpandedIds(new Set());

  const handleCreate = () => { setSelectedMenu(null); setIsFormModalOpen(true); };
  const handleEdit = (menu: MenuItem) => { setSelectedMenu(menu); setIsFormModalOpen(true); };
  const handleDelete = (menu: MenuItem) => { setSelectedMenu(menu); setIsDeleteModalOpen(true); };

  const handleFormSubmit = (data: Partial<MenuItem>) => {
    console.log('Submit:', data);
    // TODO: Add/update menu in tree
    setIsFormModalOpen(false);
    setSelectedMenu(null);
  };

  const handleDeleteConfirm = () => {
    console.log('Delete:', selectedMenu?.id);
    // TODO: Remove menu from tree
    setIsDeleteModalOpen(false);
    setSelectedMenu(null);
  };

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">
            Menu Management
          </h1>
          <p className="mt-1 text-gray-500 dark:text-gray-400">
            Manage navigation menu structure ({countMenus(menus)} total)
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="secondary" size="sm" onClick={handleExpandAll}>Expand All</Button>
          <Button variant="secondary" size="sm" onClick={handleCollapseAll}>Collapse All</Button>
          <Button onClick={handleCreate}>
            <Plus className="w-4 h-4" />
            Add Menu
          </Button>
        </div>
      </div>

      {/* Tree View */}
      <div className="bg-white dark:bg-dark-light rounded-xl shadow-sm border border-gray-200 dark:border-dark-lighter overflow-hidden">
        {/* Header */}
        <div className="px-4 py-3 bg-gray-50 dark:bg-dark-lighter border-b border-gray-200 dark:border-dark-lighter">
          <div className="flex items-center gap-2 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
            <span className="w-6"></span>
            <span className="w-6"></span>
            <span className="w-8"></span>
            <span className="flex-1">Menu Item</span>
            <span className="w-10">Level</span>
            <span className="w-8">Order</span>
            <span className="w-20">Actions</span>
          </div>
        </div>

        {/* Tree Items */}
        <div>
          {menus.map(menu => (
            <TreeItem
              key={menu.id}
              item={menu}
              depth={0}
              expandedIds={expandedIds}
              onToggle={handleToggle}
              onEdit={handleEdit}
              onDelete={handleDelete}
            />
          ))}
        </div>
      </div>

      {/* Form Modal */}
      <Modal
        isOpen={isFormModalOpen}
        onClose={() => { setIsFormModalOpen(false); setSelectedMenu(null); }}
        title={selectedMenu ? 'Edit Menu' : 'Create Menu'}
        size="md"
        closeOnClickOutside={false}
      >
        <MenuForm
          menu={selectedMenu}
          parentOptions={flattenMenus(menus).filter(m => m.id !== selectedMenu?.id)}
          onSubmit={handleFormSubmit}
          onCancel={() => { setIsFormModalOpen(false); setSelectedMenu(null); }}
        />
      </Modal>

      {/* Delete Modal */}
      <Modal
        isOpen={isDeleteModalOpen}
        onClose={() => { setIsDeleteModalOpen(false); setSelectedMenu(null); }}
        title="Confirm Delete"
        size="sm"
        closeOnClickOutside={false}
      >
        {selectedMenu && (
          <div className="space-y-4 text-center">
            <div className="flex items-center justify-center w-16 h-16 mx-auto bg-red-100 rounded-full">
              <Trash2 className="w-8 h-8 text-red-600" />
            </div>
            <p className="text-gray-500 dark:text-gray-400">
              Delete menu <strong>{selectedMenu.menu_name}</strong>?
              {selectedMenu.children && selectedMenu.children.length > 0 && (
                <span className="block text-red-500 mt-1">
                  This will also delete {selectedMenu.children.length} child menu(s)!
                </span>
              )}
            </p>
            <div className="flex justify-center gap-3">
              <Button variant="secondary" onClick={() => { setIsDeleteModalOpen(false); setSelectedMenu(null); }}>Cancel</Button>
              <Button variant="danger" onClick={handleDeleteConfirm}>Delete</Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
};

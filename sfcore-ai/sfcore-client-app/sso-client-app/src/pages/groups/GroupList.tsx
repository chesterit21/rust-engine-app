import React, { useState } from 'react';
import { Plus, Search, Edit, Trash2, Users } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';

// Mock data
const mockGroups = [
  { id: '1', name: 'Administrators', description: 'Full system access', member_count: 5, is_active: true },
  { id: '2', name: 'Managers', description: 'Department managers', member_count: 12, is_active: true },
  { id: '3', name: 'Users', description: 'Regular users', member_count: 156, is_active: true },
  { id: '4', name: 'Guests', description: 'Limited access', member_count: 8, is_active: false },
];

export const GroupList: React.FC = () => {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  const filteredGroups = mockGroups.filter((group) =>
    group.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="space-y-6 animate-fade-in">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">
            Group Management
          </h1>
          <p className="mt-1 text-gray-500 dark:text-gray-400">
            Manage user groups and permissions
          </p>
        </div>
        <Button onClick={() => setIsModalOpen(true)}>
          <Plus className="w-4 h-4" />
          Add Group
        </Button>
      </div>

      {/* Search */}
      <div className="max-w-md">
        <Input
          placeholder="Search groups..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          icon={<Search className="w-5 h-5" />}
        />
      </div>

      {/* Groups Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {filteredGroups.map((group) => (
          <div
            key={group.id}
            className="bg-white dark:bg-dark-light rounded-xl p-6 shadow-sm border border-gray-200 dark:border-dark-lighter hover:shadow-md transition-shadow"
          >
            <div className="flex items-start justify-between">
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 bg-gradient-to-br from-purple-400 to-purple-600 rounded-xl flex items-center justify-center">
                  <Users className="w-6 h-6 text-white" />
                </div>
                <div>
                  <h3 className="font-semibold text-gray-900 dark:text-white">
                    {group.name}
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    {group.description}
                  </p>
                </div>
              </div>
              <span
                className={`px-2.5 py-1 rounded-full text-xs font-medium ${
                  group.is_active
                    ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                    : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
                }`}
              >
                {group.is_active ? 'Active' : 'Inactive'}
              </span>
            </div>
            
            <div className="mt-4 pt-4 border-t border-gray-100 dark:border-dark-lighter flex items-center justify-between">
              <span className="text-sm text-gray-500 dark:text-gray-400">
                {group.member_count} members
              </span>
              <div className="flex gap-2">
                <button className="p-1.5 hover:bg-gray-100 dark:hover:bg-dark-lighter rounded-lg transition-colors">
                  <Edit className="w-4 h-4 text-gray-500" />
                </button>
                <button className="p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors">
                  <Trash2 className="w-4 h-4 text-red-500" />
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Create Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title="Create Group"
        size="md"
      >
        <form className="space-y-4">
          <Input label="Group Name" placeholder="Enter group name" />
          <Input label="Description" placeholder="Enter description" />
          <div className="flex justify-end gap-3 pt-4">
            <Button variant="secondary" onClick={() => setIsModalOpen(false)}>
              Cancel
            </Button>
            <Button type="submit">Create Group</Button>
          </div>
        </form>
      </Modal>
    </div>
  );
};

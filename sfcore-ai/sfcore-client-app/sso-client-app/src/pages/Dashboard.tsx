import React from 'react';
import { Users, UserCog, Menu, Building2, TrendingUp, Activity } from 'lucide-react';

const stats = [
  { label: 'Total Users', value: '2,543', icon: Users, color: 'bg-blue-500', change: '+12%' },
  { label: 'Active Groups', value: '48', icon: UserCog, color: 'bg-purple-500', change: '+3%' },
  { label: 'Menu Items', value: '156', icon: Menu, color: 'bg-green-500', change: '+8%' },
  { label: 'Tenants', value: '24', icon: Building2, color: 'bg-orange-500', change: '+2%' },
];

export const Dashboard: React.FC = () => {
  return (
    <div className="space-y-6 animate-fade-in">
      {/* Page Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white font-display">
          Dashboard
        </h1>
        <p className="mt-1 text-gray-500 dark:text-gray-400">
          Welcome back! Here's an overview of your SSO system.
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat) => (
          <div
            key={stat.label}
            className="bg-white dark:bg-dark-light rounded-xl p-6 shadow-sm border border-gray-200 dark:border-dark-lighter hover:shadow-md transition-shadow"
          >
            <div className="flex items-start justify-between">
              <div>
                <p className="text-sm text-gray-500 dark:text-gray-400">{stat.label}</p>
                <p className="mt-2 text-3xl font-bold text-gray-900 dark:text-white">
                  {stat.value}
                </p>
                <div className="mt-2 flex items-center gap-1 text-sm text-green-600">
                  <TrendingUp className="w-4 h-4" />
                  <span>{stat.change} from last month</span>
                </div>
              </div>
              <div className={`${stat.color} p-3 rounded-xl`}>
                <stat.icon className="w-6 h-6 text-white" />
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Activity Section */}
      <div className="bg-white dark:bg-dark-light rounded-xl p-6 shadow-sm border border-gray-200 dark:border-dark-lighter">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
          <Activity className="w-5 h-5 text-primary-500" />
          Recent Activity
        </h2>
        <div className="mt-4 space-y-4">
          {[1, 2, 3, 4, 5].map((i) => (
            <div key={i} className="flex items-center gap-4 py-3 border-b border-gray-100 dark:border-dark-lighter last:border-0">
              <div className="w-10 h-10 bg-gray-100 dark:bg-dark-lighter rounded-full flex items-center justify-center">
                <Users className="w-5 h-5 text-gray-500" />
              </div>
              <div className="flex-1">
                <p className="text-sm text-gray-900 dark:text-white">
                  New user <span className="font-medium">john.doe@example.com</span> registered
                </p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {i} hour{i > 1 ? 's' : ''} ago
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

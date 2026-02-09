import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { api } from '../api/client';
import { entities } from '../utils/entityConfig';
import { Plus, Edit2, Trash2 } from 'lucide-react';
import EntityFormModal from '../components/EntityFormModal';

const EntityListPage = () => {
  const { slug } = useParams<{ slug: string }>();
  const entityConfig = entities.find(e => e.slug === slug);
  const [data, setData] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Modal State
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<any | null>(null);

  const slugToName = (slug: string) => {
      return slug.split('-').map(word => word.charAt(0).toUpperCase() + word.slice(1)).join(' ');
  };

  useEffect(() => {
    if (!entityConfig?.endpoint) return;

    setLoading(true);
    setError(null);
    api.get(entityConfig.endpoint)
      .then((res: any) => setData(res.data))
      .catch((err: any) => {
        console.error(err);
        setError("Failed to load data");
      })
      .finally(() => setLoading(false));
  }, [slug, entityConfig]);

  const handleDelete = async (id: string) => {
    if (!confirm("Are you sure?")) return;
    try {
      await api.delete(`${entityConfig?.endpoint}/${id}`);
      setData(prev => prev.filter((item: any) => item.id !== id));
    } catch (err) {
      alert("Failed to delete");
    }
  };

  const handleSave = async (itemData: any) => {
      try {
          if (editingItem) {
              // Update
              const { data: updated } = await api.put(`${entityConfig?.endpoint}/${editingItem.id}`, itemData);
              setData(prev => prev.map(p => p.id === editingItem.id ? updated : p));
          } else {
              // Create
              const { data: created } = await api.post(entityConfig?.endpoint!, itemData);
              setData(prev => [...prev, created]);
          }
      } catch (err: any) {
          alert("Failed to save: " + err.message);
      }
  };

  if (!entityConfig) return <div className="p-8 text-slate-400">Entity not found</div>;

  return (
    <div className="p-8">
      <div className="flex justify-between items-center mb-6">
        <div>
           <h1 className="text-2xl font-bold text-white bg-gradient-to-r from-neon-cyan to-neon-teal bg-clip-text text-transparent">{slugToName(slug!)}</h1>
           <p className="text-slate-400 text-sm mt-1">Manage {slugToName(slug!).toLowerCase()} entries</p>
        </div>
        
        <button 
          onClick={() => { setEditingItem(null); setIsModalOpen(true); }}
          className="flex items-center gap-2 bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg font-bold px-4 py-2 rounded shadow-[0_0_15px_rgba(0,240,255,0.4)] hover:shadow-[0_0_25px_rgba(0,240,255,0.6)] transition"
        >
          <Plus size={18} />
          <span>New Entry</span>
        </button>
      </div>

      <div className="bg-dark-card rounded shadow overflow-hidden border-y border-white/5">
        {error && (
            <div className="bg-red-900/20 text-red-400 p-4 text-center border-b border-red-900/50">
                {error}
            </div>
        )}
        {loading ? (
           <div className="text-center py-10 text-neon-cyan animate-pulse">Loading data...</div>
        ) : data.length === 0 ? (
           <div className="text-center py-10 text-slate-500">
              <p>No records found for this entity.</p>
           </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-white/5">
              <thead className="bg-dark-bg">
                <tr>
                   <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase tracking-wider">ID</th>
                   <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase tracking-wider">Name</th>
                   <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase tracking-wider">Description</th>
                   <th className="px-6 py-3 text-right text-xs font-medium text-[#16f9f9] uppercase tracking-wider">Actions</th>
                </tr>
              </thead>
              <tbody className="bg-dark-card divide-y divide-white/5">
                {data.map((item: any) => (
                  <tr key={item.id} className="hover:bg-dark-bg/50 transition">
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-slate-500 font-mono">{item.id.substring(0,8)}...</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-white">{item.name || "Unnamed"}</td>
                    <td className="px-6 py-4 text-sm text-slate-300 max-w-md truncate">{item.description || "-"}</td>
                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                      <button 
                        onClick={() => { setEditingItem(item); setIsModalOpen(true); }}
                        className="text-neon-cyan hover:text-white mr-4 transition"
                      >
                         <Edit2 size={16} />
                      </button>
                      <button 
                        onClick={() => handleDelete(item.id)}
                        className="text-red-500 hover:text-red-400 transition"
                      >
                         <Trash2 size={16} />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {isModalOpen && (
        <EntityFormModal 
           isOpen={isModalOpen}
           initialData={editingItem}
           onClose={() => setIsModalOpen(false)}
           onSubmit={handleSave}
           entityName={slugToName(slug!)}
        />
      )}
    </div>
  );
};

export default EntityListPage;

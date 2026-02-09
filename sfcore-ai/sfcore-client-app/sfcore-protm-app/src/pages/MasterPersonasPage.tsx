import { useState, useEffect } from 'react';
import { api } from '../api/client'; // We'll add masterPersonas endpoint to client.ts later or use raw string
import { Plus, Search, Edit2, Trash2, X, Save, AlertCircle } from 'lucide-react';

interface MasterPersona {
  id: string;
  name: string;
  description: string;
  created_at?: string;
}

const MasterPersonasPage = () => {
  const [personas, setPersonas] = useState<MasterPersona[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState('');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingPersona, setEditingPersona] = useState<MasterPersona | null>(null);
  const [formData, setFormData] = useState({ name: '', description: '' });
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchPersonas();
  }, []);

  const fetchPersonas = async () => {
      setLoading(true);
      try {
        // Using direct string for now, will update client.ts later or assuming it works if I map it
        const res = await api.get('/master-personas');
        setPersonas(res.data);
      } catch (err) {
        console.error("Failed to fetch personas", err);
        setError("Failed to load personas");
      } finally {
        setLoading(false);
      }
  };

  const handleSave = async () => {
      if (!formData.name || !formData.description) {
          alert("Name and Description are required");
          return;
      }

      try {
          if (editingPersona) {
              await api.put(`/master-personas/${editingPersona.id}`, formData);
          } else {
              await api.post('/master-personas', formData);
          }
          fetchPersonas();
          setIsModalOpen(false);
          setEditingPersona(null);
          setFormData({ name: '', description: '' });
      } catch (err) {
          console.error("Failed to save persona", err);
          alert("Failed to save persona");
      }
  };

  const handleDelete = async (id: string) => {
      if (!confirm("Are you sure you want to delete this persona?")) return;
      try {
          await api.delete(`/master-personas/${id}`);
          fetchPersonas();
      } catch (err) {
          console.error("Failed to delete persona", err);
      }
  };

  const openModal = (persona: MasterPersona | null = null) => {
      if (persona) {
          setEditingPersona(persona);
          setFormData({ name: persona.name, description: persona.description });
      } else {
          setEditingPersona(null);
          setFormData({ name: '', description: '' });
      }
      setIsModalOpen(true);
  };

  const filteredPersonas = personas.filter(p => 
      p.name.toLowerCase().includes(search.toLowerCase()) || 
      p.description.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="p-8 h-full flex flex-col">
      <div className="flex justify-between items-center mb-6">
        <div>
           <h1 className="text-3xl font-bold bg-gradient-to-r from-neon-cyan via-blue-500 to-purple-600 bg-clip-text text-transparent">
             Master Personas
           </h1>
           <p className="text-slate-400 mt-1">Manage user personas for your projects</p>
        </div>
        <button 
          onClick={() => openModal()}
          className="bg-gradient-to-r from-crimson to-red-600 hover:from-red-600 hover:to-crimson text-white px-4 py-2 rounded-lg flex items-center gap-2 shadow-lg hover:shadow-crimson/50 transition-all"
        >
          <Plus size={18} /> Add Persona
        </button>
      </div>

      <div className="mb-6 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={20} />
          <input 
            type="text" 
            placeholder="Search personas..." 
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full bg-dark-card border border-dark-border rounded-xl py-3 pl-10 pr-4 text-slate-200 focus:outline-none focus:border-neon-cyan focus:shadow-[0_0_10px_rgba(0,240,255,0.2)] transition-all"
          />
      </div>

      {error && (
        <div className="bg-red-500/10 border border-red-500/50 text-red-500 p-4 rounded-xl mb-6 flex items-center gap-2">
            <AlertCircle size={20} />
            {error}
        </div>
      )}

      {loading ? (
          <div className="text-center text-slate-500 py-10">Loading...</div>
      ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 overflow-y-auto pb-10">
              {filteredPersonas.map((persona) => (
                  <div key={persona.id} className="bg-dark-card border border-dark-border rounded-xl p-5 hover:border-neon-cyan/50 transition-all group relative overflow-hidden">
                      <div className="absolute top-0 left-0 w-1 h-full bg-gradient-to-b from-crimson to-purple-600 opacity-0 group-hover:opacity-100 transition-opacity"></div>
                      <div className="flex justify-between items-start mb-2">
                          <h3 className="text-xl font-semibold text-white group-hover:text-neon-cyan transition-colors">{persona.name}</h3>
                          <div className="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                              <button onClick={() => openModal(persona)} className="p-1.5 hover:bg-white/10 rounded-lg text-yellow-400 transition-colors">
                                  <Edit2 size={16} />
                              </button>
                              <button onClick={() => handleDelete(persona.id)} className="p-1.5 hover:bg-white/10 rounded-lg text-red-500 transition-colors">
                                  <Trash2 size={16} />
                              </button>
                          </div>
                      </div>
                      <p className="text-slate-400 text-sm line-clamp-3 leading-relaxed">
                          {persona.description}
                      </p>
                  </div>
              ))}
              
              {!loading && filteredPersonas.length === 0 && (
                  <div className="col-span-full text-center py-10 text-slate-500 border border-dashed border-dark-border rounded-xl">
                      <AlertCircle className="mx-auto mb-2 opacity-50" />
                      <p>No personas found. Create one to get started!</p>
                  </div>
              )}
          </div>
      )}

      {isModalOpen && (
          <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
              <div className="bg-dark-card border border-dark-border rounded-2xl w-full max-w-md shadow-2xl transform transition-all scale-100">
                  <div className="p-6 border-b border-dark-border flex justify-between items-center">
                      <h2 className="text-xl font-bold text-white">
                          {editingPersona ? 'Edit Persona' : 'New Persona'}
                      </h2>
                      <button onClick={() => setIsModalOpen(false)} className="text-slate-400 hover:text-white">
                          <X size={24} />
                      </button>
                  </div>
                  <div className="p-6 space-y-4">
                      <div>
                          <label className="block text-sm font-medium text-slate-400 mb-1">Name</label>
                          <input 
                              type="text" 
                              value={formData.name}
                              onChange={(e) => setFormData({...formData, name: e.target.value})}
                              className="w-full bg-dark-bg border border-dark-border rounded-lg px-4 py-2 text-white focus:outline-none focus:border-neon-cyan transition-colors"
                              placeholder="e.g. End User, Administrator"
                          />
                      </div>
                      <div>
                          <label className="block text-sm font-medium text-slate-400 mb-1">Description</label>
                          <textarea 
                              value={formData.description}
                              onChange={(e) => setFormData({...formData, description: e.target.value})}
                              className="w-full bg-dark-bg border border-dark-border rounded-lg px-4 py-2 text-white focus:outline-none focus:border-neon-cyan transition-colors h-32 resize-none"
                              placeholder="Describe the persona's role, goals, and pain points..."
                          />
                      </div>
                  </div>
                  <div className="p-6 border-t border-dark-border flex justify-end gap-3">
                      <button 
                          onClick={() => setIsModalOpen(false)}
                          className="px-4 py-2 rounded-lg text-slate-300 hover:text-white hover:bg-white/5 transition-colors"
                      >
                          Cancel
                      </button>
                      <button 
                          onClick={handleSave}
                          className="bg-neon-cyan/90 hover:bg-neon-cyan text-black font-semibold px-4 py-2 rounded-lg flex items-center gap-2 shadow-[0_0_10px_rgba(0,240,255,0.3)] transition-all"
                      >
                          <Save size={18} /> Save Persona
                      </button>
                  </div>
              </div>
          </div>
      )}
    </div>
  );
};

export default MasterPersonasPage;

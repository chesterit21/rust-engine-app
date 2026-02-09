import { useEffect, useState } from 'react';
import { api } from '../api/client';
import { Plus, Edit2, Trash2, Search, Sparkles, Copy, Check, ChevronDown, ChevronRight, LayoutTemplate } from 'lucide-react';
import RightDrawer from '../components/RightDrawer';
import SearchableSelect from '../components/SearchableSelect';
import PatternTreeView from '../components/PatternTreeView';
import { NotificationToast, useNotifications } from '../components/NotificationToast';

// Helper to build tree structure from flat list
const buildTree = (items: any[]) => {
    const map = new Map();
    const roots: any[] = [];

    // Initialize map
    items.forEach(item => {
        map.set(item.id, { ...item, children: [] });
    });

    // Build hierarchy
    items.forEach(item => {
        const node = map.get(item.id);
        if (item.parent_id && map.has(item.parent_id)) {
            map.get(item.parent_id).children.push(node);
        } else {
            roots.push(node);
        }
    });

    return roots;
};

const PatternGroupRow = ({ group, onDeleteRefresh }: { group: any, onDeleteRefresh: () => void }) => {
    const [isExpanded, setIsExpanded] = useState(false);
    const [items, setItems] = useState<any[]>([]);
    const [loadingDetails, setLoadingDetails] = useState(false);
    
    // Fetch items when expanding for the first time
    const handleExpand = async () => {
        if (!isExpanded && items.length === 0) {
            setLoadingDetails(true);
            try {
                const res = await api.get(`/architecture-patterns?stack_id=${group.stack_id}&version=${group.version}`);
                setItems(res.data);
            } catch (error) {
                console.error("Failed to fetch pattern details:", error);
            } finally {
                setLoadingDetails(false);
            }
        }
        setIsExpanded(!isExpanded);
    };

    const treeRoots = buildTree(items);

    return (
        <>
            <tr 
                className={`hover:bg-dark-bg/50 transition cursor-pointer ${isExpanded ? 'bg-white/5' : ''}`}
                onClick={handleExpand}
            >
                <td className="px-6 py-4 text-sm font-medium text-white flex items-center gap-2">
                     <span className={`transition-transform ${isExpanded ? 'rotate-90' : ''}`}>
                         <ChevronRight size={16} className="text-slate-500" />
                     </span>
                     <span className="text-lg">{group.stack_name}</span>
                </td>
                <td className="px-6 py-4 text-sm text-slate-300">
                     <span className="bg-indigo-500/10 border border-indigo-500/30 text-indigo-400 px-2 py-1 rounded text-xs font-mono">
                         {group.stack_type}
                     </span>
                </td>
                <td className="px-6 py-4 text-sm">
                    <span className="text-neon-cyan font-bold bg-neon-cyan/10 border border-neon-cyan/20 px-2 py-1 rounded text-xs">
                        {group.version}
                    </span>
                </td>
                <td className="px-6 py-4 text-sm text-slate-400">
                    {group.item_count} Items
                </td>
                <td className="px-6 py-4 text-right text-sm">
                    {/* Placeholder for future group actions */}
                </td>
            </tr>
            {isExpanded && (
                <tr>
                    <td colSpan={5} className="px-0 py-0 border-b border-white/5 bg-[#0a0f1e]">
                        <div className="p-4 pl-12">
                             <div className="mb-2 text-xs text-slate-500 uppercase tracking-widest font-bold">Architecture Structure</div>
                             <div className="border border-white/5 rounded-lg bg-[#05080f] p-2">
                                {loadingDetails ? (
                                    <div className="text-slate-500 p-4 text-center italic">Loading structure...</div>
                                ) : (
                                    <PatternTreeView nodes={treeRoots} />
                                )}
                             </div>
                        </div>
                    </td>
                </tr>
            )}
        </>
    );
};

const PREDEFINED_VERSIONS = ['LITE', 'STANDAR', 'PRODUCTION GRADE'];

const ArchitecturePatternsPage = () => {
  const [groups, setGroups] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [stacks, setStacks] = useState<any[]>([]);
  
  // Notification Hook
  const { notifications, addNotification, removeNotification } = useNotifications();

  // Drawer State
  const [isDrawerOpen, setIsDrawerOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<any | null>(null);

  // Form State
  const [formData, setFormData] = useState({
    stack_id: '',
    version: 'STANDAR',
    generated_prompt: '',
    json_input: ''
  });

  const [isCopied, setIsCopied] = useState(false);

  const handleCopy = () => {
    if (!formData.generated_prompt) return;
    navigator.clipboard.writeText(formData.generated_prompt);
    setIsCopied(true);
    setTimeout(() => setIsCopied(false), 2000);
    addNotification('success', 'Copied!', 'Prompt copied to clipboard.');
  };

  // Fetch initial data
  useEffect(() => {
    fetchGroups();
    fetchStacks();
  }, []);

  const fetchGroups = () => {
    setLoading(true);
    api.get('/architecture-patterns/groups')
      .then((res: any) => setGroups(res.data))
      .catch((err: any) => {
          // Clean up error message
          const msg = err.response?.data?.message || err.message;
          addNotification('error', 'Fetch Failed', msg);
      })
      .finally(() => setLoading(false));
  };

  const fetchStacks = () => {
    api.get('/tech-stacks')
      .then((res: any) => setStacks(res.data));
  };

  // Generate Prompt Logic
  const handleGeneratePrompt = async () => {
    if (!formData.stack_id || !formData.version) {
        addNotification('error', 'Missing Input', 'Please select Tech Stack and Version first.');
        return;
    }
    
    setLoading(true);
    const notifId = addNotification('loading', 'Generating Prompt', 'Please wait while we generate the prompt...', 10000);
    
    try {
        const res = await api.post('/prompts/generate', {
            stack_id: formData.stack_id,
            version: formData.version
        });
        setFormData(prev => ({ ...prev, generated_prompt: res.data }));
        removeNotification(notifId);
        addNotification('success', 'Prompt Generated', 'AI Prompt is ready. Copy it below.');
    } catch (err: any) {
        removeNotification(notifId);
        addNotification('error', 'Generation Failed', err.message);
    } finally {
        setLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
        if (!formData.json_input) {
            addNotification('error', 'Missing JSON', 'Please paste the AI Result JSON.');
            return;
        }

        let payload;
        try {
            payload = JSON.parse(formData.json_input);
        } catch (e) {
            addNotification('error', 'Invalid JSON', 'The text provided is not valid JSON.');
            return;
        }

        // Support both single object and array
        let dataToSend = Array.isArray(payload) ? payload : [payload];
        
        // DEBUG: Show import context
        addNotification('info', 'Debug Info', `Importing: ${dataToSend.length} items. Form Stack ID: ${formData.stack_id || 'None'}`, 5000);

        const notifId = addNotification('loading', 'Importing Patterns', `Processing ${dataToSend.length} items...`, 20000);

        // Sanitize data: ensure order_index is a number and handle "N/A"
        dataToSend = dataToSend.map((item: any) => {
            let orderIdx = parseInt(item.order_index);
            if (isNaN(orderIdx)) {
                orderIdx = 0; // Default to 0 if not a number (e.g., "N/A")
            }
            
            // Sanitize parent_id: handle "NONE", "N/A", empty string
            let parentId = item.parent_id;
            if (!parentId || parentId === "NONE" || parentId === "N/A") {
                parentId = null;
            }

            return {
                ...item,
                // Prioritize stack_id from JSON, fallback to form selection
                stack_id: item.stack_id || formData.stack_id,
                parent_id: parentId, 
                order_index: orderIdx
            };
        });

        await api.post('/architecture-patterns/bulk', dataToSend);
        
        setIsDrawerOpen(false);
        fetchGroups(); // Refresh groups
        removeNotification(notifId);
        addNotification('success', 'Import Successful', `${dataToSend.length} patterns have been imported.`);
        
    } catch (err: any) {
        addNotification('error', 'Import Failed', err.response?.data || err.message);
    }
  };

  const handleDelete = async (id: string, groupKey: string) => {
      // Replaced confirm with toast? No, delete usually needs explicit confirm.
      // We can use a custom modal but for now `confirm` is blocking which is safer for delete.
      if(!confirm("Are you sure you want to delete this pattern?")) return;
      
      try {
          await api.delete(`/architecture-patterns/${id}`);
          fetchGroups();
          addNotification('success', 'Deleted', 'Pattern deleted successfully.');
      } catch (err: any) {
          addNotification('error', 'Delete Failed', err.message);
      }
  };

  return (
    <div className="p-8">
       <NotificationToast notifications={notifications} removeNotification={removeNotification} />

       {/* List View ... */}
       <div className="flex justify-between items-center mb-6">
        <div>
           <h1 className="text-2xl font-bold text-white bg-gradient-to-r from-neon-cyan to-neon-teal bg-clip-text text-transparent">
             Architecture Patterns
           </h1>
           <p className="text-slate-400 text-sm mt-1">Define standards for different tech stacks</p>
        </div>
        
        <button 
          onClick={() => { setEditingItem(null); setIsDrawerOpen(true); }}
          className="flex items-center gap-2 bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg font-bold px-4 py-2 rounded shadow-[0_0_15px_rgba(0,240,255,0.4)] hover:shadow-[0_0_25px_rgba(0,240,255,0.6)] transition"
        >
          <Plus size={18} />
          <span>New Entry</span>
        </button>
      </div>

      <div className="bg-dark-card rounded shadow overflow-hidden border-y border-white/5">
         <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-white/5">
               <thead className="bg-dark-bg">
                 <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase">Stack Name</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase">Stack Type</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase">Version</th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-[#16f9f9] uppercase">Items</th>
                    <th className="px-6 py-3 text-right text-xs font-medium text-[#16f9f9] uppercase">Actions</th>
                 </tr>
               </thead>
               <tbody className="bg-dark-card divide-y divide-white/5">
                  {groups.map((group) => (
                      <PatternGroupRow 
                          key={`${group.stack_id}-${group.version}-${group.type}`} 
                          group={group} 
                          onDeleteRefresh={fetchGroups}
                      />
                  ))}
                  {groups.length === 0 && !loading && (
                      <tr>
                          <td colSpan={5} className="px-6 py-8 text-center text-slate-500">
                              No patterns found. Create one to get started.
                          </td>
                      </tr>
                  )}
               </tbody>
            </table>
         </div>
      </div>

      <RightDrawer 
        isOpen={isDrawerOpen} 
        onClose={() => setIsDrawerOpen(false)}
        title={editingItem ? 'Edit Pattern' : 'Create Architecture Pattern'}
      >
        <form onSubmit={handleSubmit} className="space-y-6">
            
            {/* 1. Select Inputs */}
            <div className="mb-4">
                <SearchableSelect 
                     label="Tech Stack"
                     placeholder="Search & Select Tech Stack..."
                     options={stacks.map(s => ({
                         id: s.id, 
                         label: s.name, 
                         subLabel: s.language 
                     }))}
                     value={formData.stack_id}
                     onChange={(val) => setFormData({...formData, stack_id: val})}
                />
            </div>

            <div>
                 <label className="block text-sm font-medium text-slate-400 mb-1">Version/Scale</label>
                 <div className="grid grid-cols-3 gap-2">
                    {PREDEFINED_VERSIONS.map(ver => (
                        <button
                            type="button"
                            key={ver}
                            className={`px-2 py-2 text-xs font-bold rounded border transition
                                ${formData.version === ver 
                                    ? 'bg-neon-cyan/20 border-neon-cyan text-neon-cyan' 
                                    : 'bg-dark-bg border-white/10 text-slate-400 hover:border-slate-500'}
                            `}
                            onClick={() => setFormData({...formData, version: ver})}
                        >
                            {ver}
                        </button>
                    ))}
                 </div>
            </div>

            {/* 2. Generate Prompt Section */}
            <div className="pt-4 border-t border-white/5">
                <div className="flex justify-between items-center mb-2">
                    <label className="text-sm font-medium text-[#16f9f9]">Step 1: AI Prompt</label>
                    <div className="flex gap-2">
                        {formData.generated_prompt && (
                             <button
                                type="button"
                                onClick={handleCopy}
                                className="flex items-center gap-1 text-xs bg-dark-bg/80 border border-white/10 text-slate-400 hover:text-white hover:border-white/30 px-3 py-1.5 rounded transition"
                                title="Copy to clipboard"
                            >
                                {isCopied ? <Check size={12} className="text-green-400" /> : <Copy size={12} />}
                                <span>{isCopied ? 'Copied' : 'Copy'}</span>
                            </button>
                        )}
                        <button 
                            type="button"
                            onClick={handleGeneratePrompt}
                            disabled={!formData.stack_id}
                            className="flex items-center gap-1 text-xs bg-purple-600 hover:bg-purple-500 text-white px-3 py-1.5 rounded transition disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            <Sparkles size={12} />
                            Generate Prompt
                        </button>
                    </div>
                </div>
                <p className="text-xs text-slate-500 mb-2">Copy this prompt to your AI tool to generate the architecture JSON.</p>
                <textarea 
                    rows={6}
                    readOnly
                    className="w-full bg-[#05080f] border border-white/10 rounded px-3 py-2 text-slate-300 focus:border-purple-500 focus:outline-none text-xs font-mono leading-relaxed resize-none"
                    placeholder="Click 'Generate Prompt'..."
                    value={formData.generated_prompt}
                />
            </div>

            {/* 3. JSON Input Section */}
            <div className="pt-4 border-t border-white/5">
                <label className="block text-sm font-medium text-[#16f9f9] mb-1">Step 2: Paste AI Result (JSON)</label>
                <p className="text-xs text-slate-500 mb-2">Paste the JSON array output from the AI here.</p>
                <textarea 
                    rows={8}
                    required
                    className="w-full bg-dark-bg border border-white/10 rounded px-3 py-2 text-white focus:border-neon-cyan focus:outline-none placeholder-slate-600 font-mono text-xs"
                    placeholder='[ { "id": "...", "name": "...", ... } ]'
                    value={formData.json_input}
                    onChange={e => setFormData({...formData, json_input: e.target.value})}
                />
            </div>

            <div className="pt-6 flex justify-end gap-3">
                <button 
                    type="button" 
                    onClick={() => setIsDrawerOpen(false)}
                    className="px-4 py-2 rounded text-slate-400 hover:text-white transition"
                >
                    Cancel
                </button>
                <button 
                    type="submit"
                    className="px-6 py-2 rounded bg-gradient-to-r from-neon-cyan to-neon-teal text-dark-bg font-bold shadow-[0_0_15px_rgba(0,240,255,0.3)] hover:shadow-[0_0_20px_rgba(0,240,255,0.5)] transition"
                >
                    Import Patterns
                </button>
            </div>

        </form>
      </RightDrawer>
    </div>
  );
};

export default ArchitecturePatternsPage;

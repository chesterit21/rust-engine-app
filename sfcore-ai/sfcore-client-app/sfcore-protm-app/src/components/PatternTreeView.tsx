
import React, { useState } from 'react';
import { 
    Folder, 
    FileCode, 
    ChevronRight, 
    ChevronDown, 
    Box, 
    Layers, 
    Layout, 
    Database, 
    Server, 
    Globe 
} from 'lucide-react';

interface PatternNode {
    id: string;
    name: string;
    type: string; // FE, BE, FULLSTACK
    layer_rules?: string;
    naming_conventions?: string;
    children?: PatternNode[];
    order_index: number;
}

interface PatternTreeViewProps {
    nodes: PatternNode[];
    level?: number;
}

const getIcon = (name: string, type: string, isFolder: boolean) => {
    const lowerName = name.toLowerCase();
    
    // Specific icons based on name/context
    if (lowerName.includes('database') || lowerName.includes('db')) return <Database size={14} className="text-emerald-400" />;
    if (lowerName.includes('api') || lowerName.includes('server')) return <Server size={14} className="text-blue-400" />;
    if (lowerName.includes('ui') || lowerName.includes('view') || lowerName.includes('page')) return <Layout size={14} className="text-pink-400" />;
    if (lowerName.includes('utils') || lowerName.includes('helper')) return <Box size={14} className="text-yellow-400" />;
    if (lowerName.includes('hook')) return <Layers size={14} className="text-orange-400" />;
    
    // Generic Folder/File
    if (isFolder) return <Folder size={14} className="text-blue-300 fill-blue-500/20" />;
    return <FileCode size={14} className="text-slate-400" />;
};

const TreeNode: React.FC<{ node: PatternNode; level: number }> = ({ node, level }) => {
    const [isExpanded, setIsExpanded] = useState(true);
    const hasChildren = node.children && node.children.length > 0;

    // Determine if "folder" conceptually (has children or specific naming conventions implies directory)
    // Simply: if it has children, it's a folder. If not, check if it doesn't have an extension? 
    // For now, hasChildren is a good proxy, or if no extension in name.
    const isFolder = hasChildren || !node.name.includes('.');

    return (
        <div className="select-none">
            <div 
                className={`flex items-center gap-2 py-1.5 px-2 hover:bg-white/5 rounded cursor-pointer transition-colors
                    ${level === 0 ? 'bg-white/5 mb-1' : ''}
                `}
                style={{ paddingLeft: `${level * 16 + 8}px` }}
                onClick={() => setIsExpanded(!isExpanded)}
            >
                <div className="flex items-center justify-center w-4 h-4 text-slate-500">
                    {hasChildren && (
                        isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />
                    )}
                </div>

                <div className="flex-shrink-0">
                   {getIcon(node.name, node.type, isFolder)}
                </div>

                <div className="flex flex-col">
                    <span className={`text-sm truncate ${isFolder ? 'font-medium text-slate-200' : 'text-slate-400'}`}>
                        {node.name}
                    </span>
                    {level === 0 && node.layer_rules && (
                         <span className="text-[10px] text-slate-500 truncate max-w-md">{node.layer_rules}</span>
                    )}
                </div>
                
                {/* Meta Badges */}
                <div className="ml-auto flex gap-2">
                     {node.naming_conventions && (
                         <span className="text-[10px] bg-dark-bg border border-white/5 px-1.5 py-0.5 rounded text-slate-500 hidden group-hover:block">
                             {node.naming_conventions}
                         </span>
                     )}
                </div>
            </div>

            {isExpanded && hasChildren && (
                <div className="relative">
                    {/* Indentation Line */}
                    <div 
                        className="absolute left-0 top-0 bottom-0 w-px bg-white/5" 
                        style={{ left: `${level * 16 + 15}px` }} 
                    />
                    {node.children!.sort((a,b) => a.order_index - b.order_index).map(child => (
                        <TreeNode key={child.id} node={child} level={level + 1} />
                    ))}
                </div>
            )}
        </div>
    );
};

const PatternTreeView: React.FC<PatternTreeViewProps> = ({ nodes }) => {
    if (!nodes || nodes.length === 0) {
        return <div className="p-4 text-slate-500 text-sm italic">No structure defined.</div>;
    }

    // Sort roots by order_index
    const sortedNodes = [...nodes].sort((a, b) => a.order_index - b.order_index);

    return (
        <div className="flex flex-col pb-4">
            {sortedNodes.map(node => (
                <TreeNode key={node.id} node={node} level={0} />
            ))}
        </div>
    );
};

export default PatternTreeView;

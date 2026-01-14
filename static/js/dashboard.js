// Global variables
let network = null;
let currentTreeData = null;
let currentStatistics = null;

// Initialize on page load
document.addEventListener('DOMContentLoaded', async () => {
    await checkSession();
    initializeNetwork();
    setDefaultTimes();
    loadClassifications();
});

// Check if user is authenticated
async function checkSession() {
    try {
        const response = await fetch('/api/auth/check');
        const data = await response.json();

        if (!data.data || !data.data.authenticated) {
            window.location.href = '/login';
        }
    } catch (error) {
        console.error('Session check failed:', error);
        window.location.href = '/login';
    }
}

// Set default time range (1 hour ago to now)
function setDefaultTimes() {
    const now = new Date();
    const oneHourAgo = new Date(now.getTime() - 60 * 60 * 1000);

    document.getElementById('startTime').value = oneHourAgo.toISOString().slice(0, 16);
    document.getElementById('endTime').value = now.toISOString().slice(0, 16);
}

// Initialize vis-network
function initializeNetwork() {
    const container = document.getElementById('network');
    const data = { nodes: [], edges: [] };

    const options = {
        nodes: {
            shape: 'box',
            margin: 10,
            widthConstraint: { maximum: 200 },
            font: {
                size: 14,
                color: '#ffffff',
                multi: 'html'
            },
            shadow: {
                enabled: true,
                color: 'rgba(0,0,0,0.3)',
                size: 10,
                x: 2,
                y: 2
            }
        },
        edges: {
            smooth: {
                type: 'cubicBezier',
                forceDirection: 'vertical',
                roundness: 0.4
            },
            arrows: {
                to: {
                    enabled: true,
                    scaleFactor: 0.5
                }
            }
        },
        physics: {
            enabled: true,
            hierarchicalRepulsion: {
                centralGravity: 0.0,
                springLength: 150,
                springConstant: 0.01,
                nodeDistance: 180,
                damping: 0.09
            },
            solver: 'hierarchicalRepulsion',
            stabilization: {
                enabled: true,
                iterations: 1000,
                updateInterval: 25
            }
        },
        interaction: {
            hover: true,
            tooltipDelay: 100,
            navigationButtons: true,
            keyboard: true
        },
        layout: {
            hierarchical: {
                direction: 'UD',
                sortMethod: 'directed',
                nodeSpacing: 180,
                levelSeparation: 180,
                treeSpacing: 220,
                blockShifting: true,
                edgeMinimization: true,
                parentCentralization: true
            }
        }
    };

    network = new vis.Network(container, data, options);

    // Event listeners
    network.on('click', function(params) {
        if (params.nodes.length > 0) {
            const nodeId = params.nodes[0];
            showNodeDetails(nodeId);
        }
    });

    network.on('doubleClick', function(params) {
        if (params.nodes.length > 0) {
            network.focus(params.nodes[0], {
                scale: 1.5,
                animation: {
                    duration: 1000,
                    easingFunction: 'easeInOutQuad'
                }
            });
        }
    });
}

// Analyze agent
async function analyzeAgent() {
    const agentId = document.getElementById('agentId').value.trim();
    const startTime = document.getElementById('startTime').value;
    const endTime = document.getElementById('endTime').value;

    if (!agentId) {
        showAlert('Please enter an Agent ID', 'error');
        return;
    }

    if (!startTime || !endTime) {
        showAlert('Please select time range', 'error');
        return;
    }

    showLoading(true);

    try {
        const filters = getFilters();

        const response = await fetch('/api/process/analyze', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                agent_id: agentId,
                start_time: new Date(startTime).toISOString(),
                end_time: new Date(endTime).toISOString(),
                filters: filters
            })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.error || 'Analysis failed');
        }

        const data = await response.json();

        if (data.success) {
            currentTreeData = data.data.tree;
            currentStatistics = data.data.statistics;

            updateVisualization(currentTreeData);
            updateStatistics(currentStatistics);
            updateLegend(currentTreeData);

            showAlert(`Analysis complete: ${currentTreeData.total_processes} processes found`, 'success');
        } else {
            throw new Error(data.error || 'Unknown error');
        }
    } catch (error) {
        console.error('Analysis error:', error);
        showAlert(error.message, 'error');
    } finally {
        showLoading(false);
    }
}

// Get current filters
function getFilters() {
    const processName = document.getElementById('filterProcess').value.trim();
    const user = document.getElementById('filterUser').value.trim();
    const cmdContains = document.getElementById('filterCmd').value.trim();
    const excludeSystem = document.getElementById('excludeSystem').checked;

    const filters = {};
    if (processName) filters.process_name = processName;
    if (user) filters.user = user;
    if (cmdContains) filters.command_line_contains = cmdContains;
    if (excludeSystem) filters.exclude_system = true;

    return Object.keys(filters).length > 0 ? filters : null;
}

// Apply filters and re-analyze
function applyFilters() {
    analyzeAgent();
}

// Update visualization
function updateVisualization(treeData) {
    if (!network || !treeData) return;

    // Update network data
    network.setData({
        nodes: treeData.nodes,
        edges: treeData.edges
    });

    // Stabilize and fit
    network.once('stabilizationIterationsDone', function() {
        network.fit({
            animation: {
                duration: 1000,
                easingFunction: 'easeInOutQuad'
            }
        });
    });
}

// Update statistics
function updateStatistics(stats) {
    if (!stats) return;

    document.getElementById('totalProcesses').textContent = stats.total_processes || 0;
    document.getElementById('suspiciousCount').textContent = stats.suspicious_count || 0;
    document.getElementById('orphanCount').textContent = currentTreeData?.orphans?.length || 0;
}

// Update legend
function updateLegend(treeData) {
    const legendContainer = document.getElementById('legendContainer');
    if (!legendContainer || !treeData) return;

    // Count processes by classification
    const classificationCounts = {};
    treeData.nodes.forEach(node => {
        const classification = node.classification || 'Other';
        classificationCounts[classification] = (classificationCounts[classification] || 0) + 1;
    });

    // Get unique classifications with colors
    const classifications = {};
    treeData.nodes.forEach(node => {
        const classification = node.classification || 'Other';
        if (!classifications[classification]) {
            classifications[classification] = {
                name: classification,
                color: node.color.background,
                count: classificationCounts[classification]
            };
        }
    });

    // Render legend
    legendContainer.innerHTML = '';
    Object.values(classifications)
        .sort((a, b) => b.count - a.count)
        .forEach(item => {
            const legendItem = document.createElement('div');
            legendItem.className = 'legend-item';
            legendItem.innerHTML = `
                <div class="legend-color" style="background: ${item.color};"></div>
                <div class="legend-info">
                    <div class="legend-name">${item.name}</div>
                    <div class="legend-count">${item.count} processes</div>
                </div>
            `;
            legendContainer.appendChild(legendItem);
        });
}

// Show node details (could be expanded)
function showNodeDetails(nodeId) {
    if (!currentTreeData) return;

    const node = currentTreeData.nodes.find(n => n.id === nodeId);
    if (node) {
        console.log('Node details:', node);
        // Could open a modal with detailed information
    }
}

// Fit network to view
function fitNetwork() {
    if (network) {
        network.fit({
            animation: {
                duration: 1000,
                easingFunction: 'easeInOutQuad'
            }
        });
    }
}

// Export data
async function exportData(format) {
    if (!currentTreeData) {
        showAlert('No data to export', 'error');
        return;
    }

    try {
        const response = await fetch('/api/process/export', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                format: format,
                tree_data: currentTreeData
            })
        });

        if (format === 'csv') {
            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `process_tree_${new Date().getTime()}.csv`;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
        } else {
            const data = await response.json();
            const dataStr = JSON.stringify(data, null, 2);
            const blob = new Blob([dataStr], { type: 'application/json' });
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `process_tree_${new Date().getTime()}.json`;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
        }

        showAlert(`Exported as ${format.toUpperCase()}`, 'success');
    } catch (error) {
        console.error('Export error:', error);
        showAlert('Export failed', 'error');
    }
}

// Load classifications (for reference)
async function loadClassifications() {
    try {
        const response = await fetch('/api/classifications');
        const data = await response.json();

        if (data.success) {
            console.log('Classifications loaded:', data.data);
        }
    } catch (error) {
        console.error('Failed to load classifications:', error);
    }
}

// Logout
async function logout() {
    try {
        await fetch('/api/auth/logout', { method: 'POST' });
        window.location.href = '/login';
    } catch (error) {
        console.error('Logout error:', error);
        window.location.href = '/login';
    }
}

// Show alert
function showAlert(message, type) {
    const alert = document.getElementById('alert');
    alert.className = `alert alert-${type}`;
    alert.textContent = message;
    alert.style.display = 'block';

    setTimeout(() => {
        alert.style.display = 'none';
    }, 5000);
}

// Show/hide loading
function showLoading(show) {
    const loading = document.getElementById('loading');
    loading.style.display = show ? 'flex' : 'none';
}
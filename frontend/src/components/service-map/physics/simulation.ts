import type { Service, Connection, ServiceMapSettings } from '../types-exports';

interface Vector {
  x: number;
  y: number;
}

interface ForceNode extends Service {
  // Physics properties
  vx: number;
  vy: number;
  fx?: number; // fixed position x (if dragged)
  fy?: number; // fixed position y (if dragged)
  mass: number;
}

interface ForceLink {
  source: ForceNode;
  target: ForceNode;
  distance: number;
  strength: number;
  connection: Connection;
}

// Default physics settings
const DEFAULT_PHYSICS_SETTINGS = {
  enabled: true,
  gravity: 0.05,
  linkStrength: 0.7,
  friction: 0.9,
  repulsion: 300,
  centerStrength: 0.05,
  linkDistance: 100,
  // Advanced physics parameters
  decay: 0.1,      // velocity decay factor
  theta: 0.8,      // Barnes-Hut approximation accuracy
  alpha: 0.3,      // initial alpha
  alphaMin: 0.001, // minimum alpha
  alphaDecay: 0.02 // alpha decay rate
};

export class ForceSimulation {
  private nodes: ForceNode[] = [];
  private links: ForceLink[] = [];
  private settings: ServiceMapSettings['physics'];
  private width: number;
  private height: number;
  private alpha: number = 0.3;
  private iteration: number = 0;
  private isRunning: boolean = false;
  private onTick: () => void;
  
  constructor(width: number, height: number, onTick: () => void) {
    this.width = width;
    this.height = height;
    this.settings = { ...DEFAULT_PHYSICS_SETTINGS };
    this.onTick = onTick;
  }
  
  // Initialize simulation with services and connections
  public initialize(services: Service[], connections: Connection[]) {
    // Reset state
    this.nodes = [];
    this.links = [];
    this.alpha = 0.3;
    this.iteration = 0;
    
    // Create force nodes from services
    this.nodes = services.map(service => {
      // Determine mass based on service type or metrics
      let mass = 1;
      if (service.type === 'database' || service.type === 'gateway') {
        mass = 3; // Make these more stable
      } else if (service.type === 'queue') {
        mass = 2;
      }
      
      // If position is not defined, assign a random position
      const x = service.x !== undefined ? service.x : Math.random() * this.width;
      const y = service.y !== undefined ? service.y : Math.random() * this.height;
      
      return {
        ...service,
        x,
        y,
        vx: 0,
        vy: 0,
        mass
      };
    });
    
    // Create a map for quick node lookup
    const nodeMap = new Map<string, ForceNode>();
    this.nodes.forEach(node => nodeMap.set(node.id, node));
    
    // Create force links from connections
    this.links = connections
      .filter(conn => nodeMap.has(conn.source) && nodeMap.has(conn.target))
      .map(connection => {
        const source = nodeMap.get(connection.source)!;
        const target = nodeMap.get(connection.target)!;
        
        // Determine link distance and strength based on connection type
        let distance = this.settings.linkDistance;
        let strength = this.settings.linkStrength;
        
        if (connection.status === 'active') {
          strength *= 1.2; // Stronger connection for active links
        } else if (connection.status === 'idle') {
          strength *= 0.5; // Weaker connection for idle links
        }
        
        // Adjust distance based on protocol
        if (connection.protocol === 'db') {
          distance *= 0.7; // Keep database connections closer
        } else if (connection.protocol === 'http') {
          distance *= 1.2; // HTTP can be a bit longer
        }
        
        return {
          source,
          target,
          distance,
          strength,
          connection
        };
      });
    
    // Start simulation
    this.start();
  }
  
  // Update settings
  public updateSettings(settings: Partial<ServiceMapSettings['physics']>) {
    this.settings = { ...this.settings, ...settings };
  }
  
  // Resize simulation bounds
  public resize(width: number, height: number) {
    this.width = width;
    this.height = height;
  }
  
  // Start simulation
  public start() {
    if (!this.isRunning) {
      this.isRunning = true;
      this.alpha = 0.3; // Reset alpha
      requestAnimationFrame(() => this.tick());
    }
  }
  
  // Stop simulation
  public stop() {
    this.isRunning = false;
  }
  
  // Pin a node at a specific position
  public pinNode(nodeId: string, x: number, y: number) {
    const node = this.nodes.find(n => n.id === nodeId);
    if (node) {
      node.fx = x;
      node.fy = y;
      
      // Reset velocity
      node.vx = 0;
      node.vy = 0;
      
      // Ensure we're running
      this.start();
    }
  }
  
  // Release a pinned node
  public releaseNode(nodeId: string) {
    const node = this.nodes.find(n => n.id === nodeId);
    if (node) {
      node.fx = undefined;
      node.fy = undefined;
    }
  }
  
  // Main simulation tick
  private tick() {
    if (!this.isRunning || this.alpha < this.settings.alphaMin) {
      this.isRunning = false;
      return;
    }
    
    const startTime = performance.now();
    
    // Update positions with current velocities
    this.nodes.forEach(node => {
      if (node.fx !== undefined) {
        node.x = node.fx;
      } else {
        node.x += node.vx;
      }
      
      if (node.fy !== undefined) {
        node.y = node.fy;
      } else {
        node.y += node.vy;
      }
    });
    
    // Calculate forces
    this.applyForces();
    
    // Update alpha
    this.alpha = Math.max(
      this.settings.alphaMin,
      this.alpha * (1 - this.settings.alphaDecay)
    );
    
    // Call onTick callback
    this.onTick();
    
    // Continue simulation
    this.iteration++;
    requestAnimationFrame(() => this.tick());
  }
  
  // Apply all physics forces
  private applyForces() {
    // Reset velocities with decay
    this.nodes.forEach(node => {
      if (node.fx === undefined) {
        node.vx *= this.settings.friction;
      }
      if (node.fy === undefined) {
        node.vy *= this.settings.friction;
      }
    });
    
    // Apply link forces (spring-like force)
    this.applyLinkForces();
    
    // Apply repulsive forces (nodes pushing each other)
    this.applyRepulsiveForces();
    
    // Apply center gravity
    this.applyCenterForce();
    
    // Apply bounds
    this.applyBounds();
  }
  
  // Apply forces from links (connections)
  private applyLinkForces() {
    this.links.forEach(link => {
      const source = link.source;
      const target = link.target;
      
      // Skip if either node is fixed
      if ((source.fx !== undefined && source.fy !== undefined) &&
          (target.fx !== undefined && target.fy !== undefined)) {
        return;
      }
      
      // Calculate distance between nodes
      const dx = target.x - source.x;
      const dy = target.y - source.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      
      // Skip if nodes are at the same position
      if (distance === 0) return;
      
      // Calculate unit vector
      const unitX = dx / distance;
      const unitY = dy / distance;
      
      // Calculate force magnitude (spring force)
      const targetDistance = link.distance;
      const displacement = distance - targetDistance;
      const force = displacement * link.strength * this.alpha;
      
      // Apply forces to nodes (considering mass)
      if (source.fx === undefined) {
        source.vx += force * unitX / source.mass;
      }
      if (source.fy === undefined) {
        source.vy += force * unitY / source.mass;
      }
      
      if (target.fx === undefined) {
        target.vx -= force * unitX / target.mass;
      }
      if (target.fy === undefined) {
        target.vy -= force * unitY / target.mass;
      }
    });
  }
  
  // Apply repulsive forces between nodes
  private applyRepulsiveForces() {
    // This is a simplified version of n-body repulsion
    // For large graphs, we'd use Barnes-Hut or multilevel techniques
    for (let i = 0; i < this.nodes.length; i++) {
      const node1 = this.nodes[i];
      
      for (let j = i + 1; j < this.nodes.length; j++) {
        const node2 = this.nodes[j];
        
        // Calculate distance
        const dx = node2.x - node1.x;
        const dy = node2.y - node1.y;
        const distanceSq = dx * dx + dy * dy;
        
        // Avoid division by zero
        if (distanceSq === 0) continue;
        
        // Calculate repulsive force (inverse square law)
        const distance = Math.sqrt(distanceSq);
        const force = this.settings.repulsion * this.alpha / distanceSq;
        
        // Calculate force components
        const forceX = dx / distance * force;
        const forceY = dy / distance * force;
        
        // Apply forces (considering mass and fixed positions)
        if (node1.fx === undefined) {
          node1.vx -= forceX / node1.mass;
        }
        if (node1.fy === undefined) {
          node1.vy -= forceY / node1.mass;
        }
        
        if (node2.fx === undefined) {
          node2.vx += forceX / node2.mass;
        }
        if (node2.fy === undefined) {
          node2.vy += forceY / node2.mass;
        }
      }
    }
  }
  
  // Apply center gravity force
  private applyCenterForce() {
    const centerX = this.width / 2;
    const centerY = this.height / 2;
    
    this.nodes.forEach(node => {
      if (node.fx === undefined) {
        const dx = centerX - node.x;
        node.vx += dx * this.settings.centerStrength * this.alpha;
      }
      
      if (node.fy === undefined) {
        const dy = centerY - node.y;
        node.vy += dy * this.settings.centerStrength * this.alpha;
      }
    });
  }
  
  // Keep nodes within bounds
  private applyBounds() {
    const padding = 50; // Keep some padding from the edges
    const minX = padding;
    const minY = padding;
    const maxX = this.width - padding;
    const maxY = this.height - padding;
    
    this.nodes.forEach(node => {
      // Bounce off walls if we're moving out of bounds
      if (node.x < minX) {
        node.x = minX;
        if (node.vx < 0) node.vx = -node.vx * 0.5;
      } else if (node.x > maxX) {
        node.x = maxX;
        if (node.vx > 0) node.vx = -node.vx * 0.5;
      }
      
      if (node.y < minY) {
        node.y = minY;
        if (node.vy < 0) node.vy = -node.vy * 0.5;
      } else if (node.y > maxY) {
        node.y = maxY;
        if (node.vy > 0) node.vy = -node.vy * 0.5;
      }
    });
  }
  
  // Export node positions back to the original services
  public exportPositions(services: Service[]): Service[] {
    const positionMap = new Map<string, { x: number, y: number }>();
    
    this.nodes.forEach(node => {
      positionMap.set(node.id, { x: node.x, y: node.y });
    });
    
    // Update original services with new positions
    return services.map(service => {
      const position = positionMap.get(service.id);
      if (position) {
        return { ...service, x: position.x, y: position.y };
      }
      return service;
    });
  }
  
  // Apply a specific layout algorithm
  public applyLayout(layoutType: ServiceMapSettings['layout']) {
    switch (layoutType) {
      case 'circular':
        this.applyCircularLayout();
        break;
      case 'hierarchical':
        this.applyHierarchicalLayout();
        break;
      case 'concentric':
        this.applyConcentricLayout();
        break;
      case 'dagre':
        this.applyDagreLayout();
        break;
      // Force is the default and handled by the simulation itself
    }
    
    // After applying a layout, run a few iterations to stabilize
    this.alpha = 0.3;
    this.start();
  }
  
  // Circular layout
  private applyCircularLayout() {
    const centerX = this.width / 2;
    const centerY = this.height / 2;
    const radius = Math.min(this.width, this.height) * 0.4;
    
    this.nodes.forEach((node, i) => {
      const angle = (i / this.nodes.length) * 2 * Math.PI;
      node.x = centerX + radius * Math.cos(angle);
      node.y = centerY + radius * Math.sin(angle);
      node.vx = 0;
      node.vy = 0;
    });
  }
  
  // Simple hierarchical layout (tree-like)
  private applyHierarchicalLayout() {
    // Find root nodes (with no incoming connections)
    const incomingConnections = new Map<string, number>();
    
    this.links.forEach(link => {
      const targetId = link.target.id;
      incomingConnections.set(
        targetId,
        (incomingConnections.get(targetId) || 0) + 1
      );
    });
    
    const rootNodes = this.nodes.filter(
      node => !incomingConnections.has(node.id)
    );
    
    // If no root nodes, use nodes with fewest incoming connections
    if (rootNodes.length === 0) {
      let minIncoming = Infinity;
      this.nodes.forEach(node => {
        const count = incomingConnections.get(node.id) || 0;
        if (count < minIncoming) {
          minIncoming = count;
        }
      });
      
      rootNodes.push(
        ...this.nodes.filter(
          node => (incomingConnections.get(node.id) || 0) === minIncoming
        )
      );
    }
    
    // Build tree levels
    const levels: string[][] = [rootNodes.map(n => n.id)];
    const placed = new Set(levels[0]);
    
    // BFS to build levels
    let currentLevel = 0;
    while (placed.size < this.nodes.length && currentLevel < 10) {
      const nextLevel: string[] = [];
      
      levels[currentLevel].forEach(nodeId => {
        // Find all outgoing connections
        this.links
          .filter(link => link.source.id === nodeId)
          .forEach(link => {
            if (!placed.has(link.target.id)) {
              nextLevel.push(link.target.id);
              placed.add(link.target.id);
            }
          });
      });
      
      if (nextLevel.length > 0) {
        levels.push(nextLevel);
      }
      
      currentLevel++;
    }
    
    // Add any remaining nodes to the last level
    const lastLevel = levels[levels.length - 1] || [];
    this.nodes.forEach(node => {
      if (!placed.has(node.id)) {
        lastLevel.push(node.id);
      }
    });
    
    // Position nodes by level
    const levelHeight = this.height / (levels.length || 1);
    
    levels.forEach((levelNodes, levelIndex) => {
      const y = levelHeight * (levelIndex + 0.5);
      const nodeWidth = this.width / (levelNodes.length || 1);
      
      levelNodes.forEach((nodeId, nodeIndex) => {
        const node = this.nodes.find(n => n.id === nodeId);
        if (node) {
          node.x = nodeWidth * (nodeIndex + 0.5);
          node.y = y;
          node.vx = 0;
          node.vy = 0;
        }
      });
    });
  }
  
  // Concentric layout (nodes grouped by type)
  private applyConcentricLayout() {
    const centerX = this.width / 2;
    const centerY = this.height / 2;
    
    // Group nodes by type
    const nodesByType = new Map<string, ForceNode[]>();
    
    this.nodes.forEach(node => {
      if (!nodesByType.has(node.type)) {
        nodesByType.set(node.type, []);
      }
      nodesByType.get(node.type)!.push(node);
    });
    
    // Sort types by importance
    const typeOrder = [
      'gateway', 'auth', 'service', 'api', 
      'database', 'queue', 'cache', 'storage', 'external'
    ];
    
    const sortedTypes = Array.from(nodesByType.keys()).sort((a, b) => {
      const indexA = typeOrder.indexOf(a);
      const indexB = typeOrder.indexOf(b);
      
      if (indexA === -1 && indexB === -1) return 0;
      if (indexA === -1) return 1;
      if (indexB === -1) return -1;
      
      return indexA - indexB;
    });
    
    // Position nodes in concentric circles by type
    const maxRadius = Math.min(this.width, this.height) * 0.4;
    const radiusStep = maxRadius / (sortedTypes.length || 1);
    
    sortedTypes.forEach((type, typeIndex) => {
      const nodes = nodesByType.get(type) || [];
      const radius = radiusStep * (typeIndex + 1);
      
      nodes.forEach((node, i) => {
        const angle = (i / nodes.length) * 2 * Math.PI;
        node.x = centerX + radius * Math.cos(angle);
        node.y = centerY + radius * Math.sin(angle);
        node.vx = 0;
        node.vy = 0;
      });
    });
  }
  
  // Dagre-like layout (for directed acyclic graphs)
  private applyDagreLayout() {
    // This is a simplified version of dagre layout
    // In a real implementation, we would use the dagre library
    
    // Find node ranks using a topological sort-like approach
    const inDegree = new Map<string, number>();
    const nodeRanks = new Map<string, number>();
    
    // Calculate in-degree for each node
    this.nodes.forEach(node => {
      inDegree.set(node.id, 0);
    });
    
    this.links.forEach(link => {
      const targetId = link.target.id;
      inDegree.set(targetId, (inDegree.get(targetId) || 0) + 1);
    });
    
    // Nodes with in-degree 0 are at rank 0
    const queue: { id: string; rank: number }[] = [];
    this.nodes.forEach(node => {
      if ((inDegree.get(node.id) || 0) === 0) {
        nodeRanks.set(node.id, 0);
        queue.push({ id: node.id, rank: 0 });
      }
    });
    
    // Process queue
    while (queue.length > 0) {
      const { id, rank } = queue.shift()!;
      
      // Find outgoing edges
      this.links
        .filter(link => link.source.id === id)
        .forEach(link => {
          const targetId = link.target.id;
          inDegree.set(targetId, (inDegree.get(targetId) || 0) - 1);
          
          // If all incoming edges processed, assign rank
          if ((inDegree.get(targetId) || 0) === 0) {
            const newRank = rank + 1;
            nodeRanks.set(targetId, newRank);
            queue.push({ id: targetId, rank: newRank });
          }
        });
    }
    
    // Handle cycles by assigning remaining nodes to ranks
    this.nodes.forEach(node => {
      if (!nodeRanks.has(node.id)) {
        // Find minimum rank of neighbors + 1
        let minRank = Infinity;
        let found = false;
        
        this.links.forEach(link => {
          if (link.target.id === node.id && nodeRanks.has(link.source.id)) {
            minRank = Math.min(minRank, nodeRanks.get(link.source.id)! + 1);
            found = true;
          }
        });
        
        if (!found) {
          // No ranked neighbors, assign to rank 0
          nodeRanks.set(node.id, 0);
        } else {
          nodeRanks.set(node.id, minRank);
        }
      }
    });
    
    // Group nodes by rank
    const rankGroups = new Map<number, string[]>();
    this.nodes.forEach(node => {
      const rank = nodeRanks.get(node.id) || 0;
      if (!rankGroups.has(rank)) {
        rankGroups.set(rank, []);
      }
      rankGroups.get(rank)!.push(node.id);
    });
    
    // Position nodes by rank
    const ranks = Array.from(rankGroups.keys()).sort((a, b) => a - b);
    const rankHeight = this.height / (ranks.length || 1);
    
    ranks.forEach(rank => {
      const nodesInRank = rankGroups.get(rank) || [];
      const nodeWidth = this.width / (nodesInRank.length || 1);
      
      nodesInRank.forEach((nodeId, index) => {
        const node = this.nodes.find(n => n.id === nodeId);
        if (node) {
          node.x = nodeWidth * (index + 0.5);
          node.y = rankHeight * (rank + 0.5);
          node.vx = 0;
          node.vy = 0;
        }
      });
    });
  }
}
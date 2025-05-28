export interface ServiceMetrics {
  requestRate: number;
  errorRate: number;
  cpuUsage: number;
  memoryUsage: number;
  responseTime: number;
  saturation: number; // percentage of capacity
}

export interface ConnectionMetrics {
  latency: number;
  throughput: number;
  errorRate: number;
  requestCount: number;
  packetLoss: number;
}

export interface DataPacket {
  progress: number;
  speed: number;
  size: number;
  type: 'request' | 'response' | 'error';
}

export interface Service {
  id: string;
  name: string;
  type: 'api' | 'database' | 'queue' | 'cache' | 'service' | 'gateway' | 'external' | 'auth' | 'storage';
  status: 'healthy' | 'warning' | 'error' | 'unknown';
  metrics: ServiceMetrics;
  // Layout properties
  x?: number;
  y?: number;
  vx?: number; // Velocity (used in simulation)
  vy?: number;
  size?: number; // Node size for visualization
  // Rendering properties
  color?: string;
  glowIntensity?: number;
  highlighted?: boolean;
  selected?: boolean;
  // Metadata
  version?: string;
  region?: string;
  instances?: number;
  metadata?: Record<string, any>;
}

export interface Connection {
  id: string;
  source: string;
  target: string;
  status: 'active' | 'idle' | 'slow' | 'error';
  protocol: 'http' | 'grpc' | 'kafka' | 'sqs' | 'db' | 'tcp' | 'udp' | 'websocket' | 'custom';
  metrics: ConnectionMetrics;
  // Rendering properties
  dataPackets?: DataPacket[];
  width?: number;
  color?: string;
  animated?: boolean;
  highlighted?: boolean;
  selected?: boolean;
  bidirectional?: boolean;
  // Metadata
  label?: string;
  metadata?: Record<string, any>;
}

export interface ServiceMapData {
  services: Service[];
  connections: Connection[];
  timestamp: number; // Timestamp of the data snapshot
}

// WebGL and rendering specific interfaces
export interface ServiceMapSettings {
  layout: 'force' | 'hierarchical' | 'circular' | 'concentric' | 'dagre';
  showLabels: boolean;
  showMetrics: boolean;
  animateEdges: boolean;
  colorBy: 'status' | 'type' | 'latency' | 'errorRate';
  sizeBy: 'fixed' | 'requestRate' | 'errorRate' | 'instances';
  focusMode: boolean;
  physics: {
    enabled: boolean;
    gravity: number;
    linkStrength: number;
    friction: number;
    repulsion: number;
  };
  performance: {
    useWebGL: boolean;
    nodeLimit: number;
    edgeLimit: number;
    animationFPS: number;
    detailLevel: 'low' | 'medium' | 'high' | 'ultra';
  };
  theme: {
    background: string;
    text: string;
    healthyColor: string;
    warningColor: string;
    errorColor: string;
    unknownColor: string;
    selectionGlow: string;
    highlightGlow: string;
  };
}

export interface ServiceMapState {
  hoveredNode: Service | null;
  hoveredEdge: Connection | null;
  selectedNode: Service | null;
  selectedEdge: Connection | null;
  transform: {
    x: number;
    y: number;
    k: number; // zoom scale
  };
}

// WebGL-specific types
export interface ShaderProgram {
  program: WebGLProgram;
  attributes: Record<string, number>;
  uniforms: Record<string, WebGLUniformLocation>;
}

export interface GraphRenderState {
  nodePositions: Float32Array;
  nodeColors: Float32Array;
  nodeSizes: Float32Array;
  nodeGlow: Float32Array;
  edgePositions: Float32Array;
  edgeColors: Float32Array;
  edgeWidths: Float32Array;
  packetPositions: Float32Array;
  packetColors: Float32Array;
  packetSizes: Float32Array;
  highlightedNodes: Set<string>;
  highlightedEdges: Set<string>;
}

export interface GraphBuffers {
  nodePositionBuffer: WebGLBuffer;
  nodeColorBuffer: WebGLBuffer;
  nodeSizeBuffer: WebGLBuffer;
  nodeGlowBuffer: WebGLBuffer;
  edgePositionBuffer: WebGLBuffer;
  edgeColorBuffer: WebGLBuffer;
  edgeWidthBuffer: WebGLBuffer;
  packetPositionBuffer: WebGLBuffer;
  packetColorBuffer: WebGLBuffer;
  packetSizeBuffer: WebGLBuffer;
}

// Performance monitoring types
export interface PerformanceStats {
  fps: number;
  renderTime: number;
  physicsTime: number;
  nodeCount: number;
  edgeCount: number;
  frameCount: number;
  gpuMemory?: number;
}
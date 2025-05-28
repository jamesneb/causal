export interface MemoryState {
  used: number;
  total: number;
  heapUsed: number;
  heapTotal: number;
  external: number;
  arrayBuffers: number;
  gc?: {
    lastRun: number;
    totalPauses: number;
    count: number;
  };
}

export interface Environment {
  runtime?: {
    version: string;
    node: string;
  };
  os?: {
    platform: string;
    arch: string;
    version: string;
  };
  uptime?: number;
  variables?: Record<string, string>;
}

export interface Connection {
  id: string;
  type: string;
  remote: string;
  status: 'open' | 'closing' | 'closed' | 'active' | 'idle' | 'error' | 'connected' | 'waiting';
  duration: number;
  metadata?: Record<string, any>;
}

export interface ConnectionsState {
  current: number;
  total: number;
  peak: number;
  list?: Connection[];
}

export interface ServiceState {
  id: string;
  name: string;
  version: string;
  status: 'healthy' | 'warning' | 'error' | 'starting' | 'stopping' | 'unavailable';
  timestamp: number;
  memory?: MemoryState;
  environment?: Environment;
  connections?: ConnectionsState;
  metrics?: Record<string, any>;
  state?: Record<string, any>;
  config?: Record<string, any>;
}

export interface StateSnapshot {
  id: string;
  timestamp: number;
  type: 'breakpoint' | 'error' | 'manual' | 'interval';
  serviceId: string;
  state: Record<string, any>;
  changedPaths?: string[];
  metadata?: Record<string, any>;
}
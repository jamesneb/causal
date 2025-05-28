export interface Span {
  id: string;
  name: string;
  serviceName: string;
  startTime: number;  // timestamp in ms
  endTime: number;    // timestamp in ms
  duration: number;   // in ms
  status: 'success' | 'error' | 'warning';
  parentId?: string;
  attributes?: Record<string, any>;
}

export interface Trace {
  id: string;
  name: string;
  startTime: number;
  endTime: number;
  duration: number;
  status: 'success' | 'error' | 'warning';
  spans: Span[];
  attributes?: Record<string, any>;
}

export interface TimelineEvent {
  id: string;
  timestamp: number;
  name: string;
  type: 'breakpoint' | 'error' | 'warning' | 'info' | 'custom';
  serviceName: string;
  description?: string;
  attributes?: Record<string, any>;
}

export interface TimeRange {
  startTime: number;
  endTime: number;
}

export interface TimelineOptions {
  showEvents: boolean;
  showServiceNames: boolean;
  colorByService: boolean;
  colorByStatus: boolean;
  showConnections: boolean;
}
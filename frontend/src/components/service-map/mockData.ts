import type { ServiceMapData, Service, Connection, DataPacket } from './types-exports';

// Utility function to generate random value within range
function randomInRange(min: number, max: number): number {
  return Math.random() * (max - min) + min;
}

// Generate random packets for connection animation
function generateDataPackets(count: number): DataPacket[] {
  return Array.from({ length: count }, () => ({
    progress: Math.random(), // 0 to 1, position along the connection
    speed: randomInRange(0.001, 0.01),
    size: randomInRange(2, 5),
    type: Math.random() > 0.9 ? 'error' : Math.random() > 0.5 ? 'response' : 'request'
  }));
}

// Service types with realistic configurations
const serviceTypes = [
  { type: 'gateway', name: 'API Gateway', count: 1 },
  { type: 'auth', name: 'Auth Service', count: 1 },
  { type: 'service', name: 'User Service', count: 1 },
  { type: 'service', name: 'Product Service', count: 1 },
  { type: 'service', name: 'Order Service', count: 1 },
  { type: 'service', name: 'Payment Service', count: 1 },
  { type: 'service', name: 'Notification Service', count: 1 },
  { type: 'service', name: 'Analytics Service', count: 1 },
  { type: 'service', name: 'Recommendation Service', count: 1 },
  { type: 'database', name: 'User DB', count: 1 },
  { type: 'database', name: 'Product DB', count: 1 },
  { type: 'database', name: 'Order DB', count: 1 },
  { type: 'queue', name: 'Order Queue', count: 1 },
  { type: 'queue', name: 'Notification Queue', count: 1 },
  { type: 'cache', name: 'Product Cache', count: 1 },
  { type: 'cache', name: 'User Cache', count: 1 },
  { type: 'external', name: 'Payment Gateway', count: 1 },
  { type: 'external', name: 'Email Provider', count: 1 },
  { type: 'external', name: 'SMS Provider', count: 1 },
  { type: 'storage', name: 'Object Storage', count: 1 }
];

// Connection patterns that make sense for a microservice architecture
const connectionPatterns = [
  { source: 'API Gateway', target: 'Auth Service', protocol: 'http' },
  { source: 'API Gateway', target: 'User Service', protocol: 'http' },
  { source: 'API Gateway', target: 'Product Service', protocol: 'http' },
  { source: 'API Gateway', target: 'Order Service', protocol: 'http' },
  { source: 'User Service', target: 'User DB', protocol: 'db' },
  { source: 'User Service', target: 'User Cache', protocol: 'tcp' },
  { source: 'Product Service', target: 'Product DB', protocol: 'db' },
  { source: 'Product Service', target: 'Product Cache', protocol: 'tcp' },
  { source: 'Order Service', target: 'Order DB', protocol: 'db' },
  { source: 'Order Service', target: 'Payment Service', protocol: 'http' },
  { source: 'Order Service', target: 'Order Queue', protocol: 'kafka' },
  { source: 'Payment Service', target: 'Payment Gateway', protocol: 'http' },
  { source: 'Notification Service', target: 'Notification Queue', protocol: 'kafka' },
  { source: 'Notification Service', target: 'Email Provider', protocol: 'http' },
  { source: 'Notification Service', target: 'SMS Provider', protocol: 'http' },
  { source: 'Analytics Service', target: 'User DB', protocol: 'db' },
  { source: 'Analytics Service', target: 'Product DB', protocol: 'db' },
  { source: 'Analytics Service', target: 'Order DB', protocol: 'db' },
  { source: 'Recommendation Service', target: 'Product DB', protocol: 'db' },
  { source: 'Recommendation Service', target: 'Product Cache', protocol: 'tcp' },
  { source: 'Notification Queue', target: 'Notification Service', protocol: 'kafka' },
  { source: 'Order Queue', target: 'Notification Service', protocol: 'kafka' },
  { source: 'Order Service', target: 'Object Storage', protocol: 'http' },
  { source: 'User Service', target: 'Auth Service', protocol: 'grpc' }
];

// Generate a mock service
function createMockService(id: string, name: string, type: string): Service {
  // Set health status with weighted randomness
  let status: 'healthy' | 'warning' | 'error' | 'unknown';
  const healthRoll = Math.random();
  if (healthRoll > 0.95) status = 'error';
  else if (healthRoll > 0.85) status = 'warning';
  else if (healthRoll > 0.98) status = 'unknown';
  else status = 'healthy';
  
  // Generate metrics based on status
  const errorMultiplier = status === 'error' ? randomInRange(5, 10) : 
                          status === 'warning' ? randomInRange(2, 5) : 1;
                          
  const metrics = {
    requestRate: randomInRange(10, 500),
    errorRate: randomInRange(0.1, 3) * errorMultiplier,
    cpuUsage: randomInRange(5, status === 'error' ? 95 : 60),
    memoryUsage: randomInRange(10, status === 'error' ? 90 : 70),
    responseTime: randomInRange(5, status === 'error' ? 500 : 100),
    saturation: randomInRange(10, status === 'error' ? 95 : 70)
  };
  
  // Type-specific properties
  let instances = 1;
  if (type === 'service' || type === 'api') {
    instances = Math.floor(randomInRange(2, 8));
  }
  
  // Create service object
  return {
    id,
    name,
    type: type as any,
    status,
    metrics,
    version: `v${Math.floor(randomInRange(1, 4))}.${Math.floor(randomInRange(0, 10))}.${Math.floor(randomInRange(0, 20))}`,
    region: Math.random() > 0.7 ? 'us-west' : Math.random() > 0.5 ? 'us-east' : 'eu-central',
    instances
  };
}

// Generate a mock connection
function createMockConnection(id: string, source: string, target: string, protocol: string): Connection {
  // Set status based on connected services
  const statusRoll = Math.random();
  let status: 'active' | 'idle' | 'slow' | 'error';
  
  if (statusRoll > 0.95) status = 'error';
  else if (statusRoll > 0.85) status = 'slow';
  else if (statusRoll > 0.7) status = 'idle';
  else status = 'active';
  
  // Generate metrics based on status
  const latencyBase = 
    status === 'error' ? randomInRange(300, 2000) : 
    status === 'slow' ? randomInRange(100, 300) : 
    randomInRange(5, 100);
    
  const metrics = {
    latency: latencyBase,
    throughput: status === 'idle' ? randomInRange(1, 10) : randomInRange(10, 500),
    errorRate: status === 'error' ? randomInRange(5, 50) : randomInRange(0, 2),
    requestCount: Math.floor(randomInRange(100, 10000)),
    packetLoss: status === 'error' ? randomInRange(1, 10) : 0
  };
  
  // Create animated data packets
  const packetCount = status === 'idle' ? 0 : Math.floor(randomInRange(1, 10));
  
  return {
    id,
    source,
    target,
    status,
    protocol: protocol as any,
    metrics,
    dataPackets: generateDataPackets(packetCount),
    bidirectional: Math.random() > 0.7,
    animated: status !== 'idle'
  };
}

// Generate complete mock data
export function generateMockServiceMapData(): ServiceMapData {
  const services: Service[] = [];
  const serviceMap: Record<string, string> = {}; // name to id mapping
  
  // Create services
  serviceTypes.forEach(({ type, name, count }) => {
    for (let i = 0; i < count; i++) {
      const id = `service-${services.length}`;
      const serviceName = count > 1 ? `${name} ${i + 1}` : name;
      services.push(createMockService(id, serviceName, type));
      serviceMap[serviceName] = id;
    }
  });
  
  // Create connections
  const connections: Connection[] = [];
  connectionPatterns.forEach((pattern, index) => {
    if (serviceMap[pattern.source] && serviceMap[pattern.target]) {
      connections.push(
        createMockConnection(
          `conn-${index}`,
          serviceMap[pattern.source],
          serviceMap[pattern.target],
          pattern.protocol
        )
      );
    }
  });
  
  // Add some random additional connections for complexity
  for (let i = 0; i < 10; i++) {
    const sourceIndex = Math.floor(Math.random() * services.length);
    let targetIndex = Math.floor(Math.random() * services.length);
    while (targetIndex === sourceIndex) {
      targetIndex = Math.floor(Math.random() * services.length);
    }
    
    const protocols = ['http', 'grpc', 'tcp', 'db'];
    const randomProtocol = protocols[Math.floor(Math.random() * protocols.length)];
    
    connections.push(
      createMockConnection(
        `conn-extra-${i}`,
        services[sourceIndex].id,
        services[targetIndex].id,
        randomProtocol
      )
    );
  }
  
  return {
    services,
    connections,
    timestamp: Date.now()
  };
}

// Default mock data export
export const mockServiceMapData = generateMockServiceMapData();